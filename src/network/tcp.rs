//! TCP network module: Handles TCP connection logic for peer communication in the P2P Chat.
//!
//! This module is responsible for managing TCP connections with peers,
//! handling incoming messages, and broadcasting outgoing messages.
//! It utilizes Tokio's asynchronous runtime for non-blocking I/O operations.

use crate::crypto::SignedMessage;
use crate::error::ChatError;
use crate::peer::{NetworkMessage, PeerInfo};
use chrono::Utc;
use colored::*;
use serde_json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};

pub async fn handle_tcp_connection(
    stream: TcpStream,
    _addr: SocketAddr,
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    message_sender: broadcast::Sender<String>,
    peer_id: String,
    threshold_manager: Arc<crate::crypto::threshold::ThresholdManager>,
    crypto_manager: Arc<crate::crypto::CryptoManager>,
) -> Result<(), ChatError> {
    let mut buf = [0; 1024];

    while let Ok(_n) = stream.readable().await {
        match stream.try_read(&mut buf) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                if let Ok(network_msg) = serde_json::from_slice::<NetworkMessage>(&buf[..n]) {
                    println!("🔍 Received message: {network_msg:?}");
                    match network_msg {
                        NetworkMessage::Chat(message) => {
                            handlers::handle_chat_message(message, &message_sender, &crypto_manager)
                                .await
                        }
                        NetworkMessage::Exit(peer_id) => {
                            handlers::handle_exit(&peers, peer_id).await
                        }
                        NetworkMessage::Discovery(peer_info) => {
                            handlers::handle_discovery(&peers, peer_info, peer_id.clone()).await
                        }
                        NetworkMessage::Heartbeat(_) => {
                            handlers::handle_heartbeat().await;
                        }
                        NetworkMessage::SignedChat(signed_message) => {
                            handlers::handle_signed_chat(
                                signed_message,
                                &message_sender,
                                &crypto_manager,
                            )
                            .await
                        }
                        NetworkMessage::IdentityAnnouncement {
                            peer_id,
                            name,
                            public_key,
                        } => {
                            handlers::handle_identity_announcement(
                                peer_id,
                                name,
                                public_key,
                                &crypto_manager,
                            )
                            .await
                        }
                        NetworkMessage::UpgradeRequest(proposal) => {
                            handlers::handle_upgrade_request(
                                proposal,
                                threshold_manager.clone(),
                                &message_sender,
                            )
                            .await
                        }
                        NetworkMessage::UpgradeVote(vote) => {
                            handlers::handle_upgrade_vote(
                                vote,
                                threshold_manager.clone(),
                                &message_sender,
                            )
                            .await
                        }
                        NetworkMessage::PartialSignature(partial_sig) => {
                            handlers::handle_partial_signature(partial_sig, &message_sender).await
                        }
                    }
                }
            }
            Err(e) => return Err(ChatError::Network(e.to_string())),
        }
    }
    Ok(())
}

mod handlers {
    use crate::{
        crypto::threshold::{PartialSignature, UpgradeProposal, UpgradeVote},
        peer::Message,
    };

    use super::*;

    pub async fn handle_chat_message(
        message: Message,
        message_sender: &broadcast::Sender<String>,
        crypto_manager: &Arc<crate::crypto::CryptoManager>,
    ) {
        // Check if message has cryptographic signature
        if let (Some(signature), Some(public_key)) = (&message.signature, &message.public_key) {
            // Verify the signature if we have crypto capabilities
            println!(
                "🔍 Verifying message from {} with signature length: {}",
                message.from_name,
                signature.len()
            );

            let signed_msg = &SignedMessage {
                message: message.content.clone(),
                signature: signature.clone(),
                public_key: public_key.clone(),
                signer_id: message.from_id.clone(),
                signer_name: message.from_name.clone(),
                timestamp: message.timestamp,
            };

            _verify_and_display(signed_msg, message_sender, crypto_manager).await;
        } else {
            // No crypto manager, display as unsigned message
            let display_msg = format!(
                "📝 {} says (unsigned): {}",
                message.from_name, message.content
            );
            let _ = message_sender.send(display_msg);
        }
    }

    pub async fn handle_signed_chat(
        signed_message: SignedMessage,
        message_sender: &broadcast::Sender<String>,
        crypto_manager: &Arc<crate::crypto::CryptoManager>,
    ) {
        {
            _verify_and_display(&signed_message, message_sender, crypto_manager).await;
        }
    }

    pub async fn handle_heartbeat() {
        // TODO implement
        // Handle heartbeat messages
    }

    pub async fn handle_discovery(
        peers: &Arc<Mutex<HashMap<String, PeerInfo>>>,
        peer_info: PeerInfo,
        peer_id: String,
    ) {
        {
            if peer_info.id == peer_id {
                // Ignore our own Discovery messages
                return;
            }
            // Validate discovered peer before adding
            if !peer_info.is_valid() {
                eprintln!("Invalid peer info received via TCP: {peer_info:?}");
                return;
            }
            let mut peers = peers.lock().await;
            if !peers.contains_key(&peer_info.id) {
                println!(
                    "🔗 Discovered peer via TCP: {} at {}",
                    peer_info.name, peer_info.ip
                );
            }
            peers.insert(peer_info.id.clone(), peer_info);
        }
    }

    pub async fn handle_identity_announcement(
        peer_id: String,
        name: String,
        public_key: Vec<u8>,
        crypto_manager: &Arc<crate::crypto::CryptoManager>,
    ) {
        if let Err(e) = &crypto_manager
            .add_known_peer(peer_id.clone(), public_key.clone())
            .await
        {
            eprintln!("Failed to add peer key: {e}");
        } else {
            println!(
                "🔐 Added public key for peer {}: {}",
                name,
                hex::encode(&public_key[..8])
            );
        }
    }

    pub async fn handle_exit(peers: &Arc<Mutex<HashMap<String, PeerInfo>>>, peer_id: String) {
        let mut peers = peers.lock().await;
        if peers.remove(&peer_id).is_some() {
            let timestamp = Utc::now().format("%H:%M:%S");
            println!(
                "[{}] {} Peer {} exited and was removed from the list.",
                timestamp.to_string().dimmed(),
                "❌".bright_red(),
                peer_id.bright_yellow()
            );
        }
    }

    pub async fn handle_upgrade_request(
        proposal: UpgradeProposal,
        threshold_manager: Arc<crate::crypto::threshold::ThresholdManager>,
        message_sender: &broadcast::Sender<String>,
    ) {
        println!(
            "🔐 Received upgrade proposal from {}: {}",
            proposal.proposer_name, proposal.description
        );
        println!(
            "📊 Proposal ID: {}, requires {}/{} approvals",
            proposal.proposal_id, proposal.required_approvals, proposal.total_peers
        );

        // Store proposal locally if not present
        threshold_manager
            .insert_received_proposal(proposal.clone())
            .await;

        let display_msg = format!(
            "🔐 {} proposed secure messaging upgrade: {} (ID: {})",
            proposal.proposer_name, proposal.description, proposal.proposal_id
        );
        let _ = message_sender.send(display_msg);
    }

    pub async fn handle_upgrade_vote(
        vote: UpgradeVote,
        threshold_manager: Arc<crate::crypto::threshold::ThresholdManager>,
        message_sender: &broadcast::Sender<String>,
    ) {
        println!(
            "🗳️  Received vote from {} on proposal {}: {}",
            vote.voter_name,
            vote.proposal_id,
            if vote.approved {
                "✅ APPROVED"
            } else {
                "❌ REJECTED"
            }
        );

        // TODO: Process vote locally
        let _ = threshold_manager.handle_received_vote(&vote).await;

        let display_msg = format!(
            "🗳️  {} voted {} on upgrade proposal {}",
            vote.voter_name,
            if vote.approved {
                "✅ APPROVED"
            } else {
                "❌ REJECTED"
            },
            vote.proposal_id
        );
        let _ = message_sender.send(display_msg);
    }

    pub async fn handle_partial_signature(
        partial_sig: PartialSignature,
        message_sender: &broadcast::Sender<String>,
    ) {
        println!(
            "🔐 Received partial signature from {} on proposal {}",
            partial_sig.signer_name, partial_sig.proposal_id
        );

        // TODO: Process partial signature for threshold verification
        let display_msg = format!(
            "🔐 {} provided partial signature for proposal {}",
            partial_sig.signer_name, partial_sig.proposal_id
        );
        let _ = message_sender.send(display_msg);
    }
}

// PRIVATE HELPERS
pub fn _format_verified(name: &str, content: &str) -> String {
    format!("🔐 {name} says (verified): {content}")
}

async fn _verify_and_display(
    signed_message: &SignedMessage,
    message_sender: &broadcast::Sender<String>,
    crypto_manager: &Arc<crate::crypto::CryptoManager>,
) {
    match crypto_manager.verify_message(signed_message).await {
        Ok(true) => {
            let _ = message_sender.send(_format_verified(
                &signed_message.signer_name,
                &signed_message.message,
            ));
        }
        Ok(false) => {
            let _ = message_sender.send(format!(
                "⚠️  {} says (INVALID SIGNATURE): {}",
                signed_message.signer_name, signed_message.message
            ));
        }
        Err(e) => {
            let _ = message_sender.send(format!(
                "❓ {} says (verification failed: {}): {}",
                signed_message.signer_name, e, signed_message.message
            ));
        }
    }
}
