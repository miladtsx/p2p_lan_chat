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
    crypto_manager: Option<Arc<crate::crypto::CryptoManager>>,
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
                            // Check if message has cryptographic signature
                            if let (Some(signature), Some(public_key)) =
                                (&message.signature, &message.public_key)
                            {
                                if let Some(crypto_mgr) = &crypto_manager {
                                    // Verify the signature if we have crypto capabilities
                                    println!("🔍 Verifying message from {} with signature length: {}", 
                                        message.from_name, signature.len());
                                    match crypto_mgr
                                        .verify_message(&SignedMessage {
                                            message: message.content.clone(),
                                            signature: signature.clone(),
                                            public_key: public_key.clone(),
                                            signer_id: message.from_id.clone(),
                                            signer_name: message.from_name.clone(),
                                            timestamp: message.timestamp,
                                        })
                                        .await
                                    {
                                        Ok(true) => {
                                            println!("✅ Message verification successful!");
                                            let display_msg = format!(
                                                "🔐 {} says (verified): {}",
                                                message.from_name, message.content
                                            );
                                            let _ = message_sender.send(display_msg);
                                        }
                                        Ok(false) => {
                                            let display_msg = format!(
                                                "⚠️  {} says (INVALID SIGNATURE): {}",
                                                message.from_name, message.content
                                            );
                                            let _ = message_sender.send(display_msg);
                                        }
                                        Err(_) => {
                                            let display_msg = format!(
                                                "❓ {} says (verification failed): {}",
                                                message.from_name, message.content
                                            );
                                            let _ = message_sender.send(display_msg);
                                        }
                                    }
                                } else {
                                    // No crypto manager, display as unsigned message
                                    let display_msg =
                                        format!("📝 {} says (unsigned): {}", message.from_name, message.content);
                                    let _ = message_sender.send(display_msg);
                                }
                            } else {
                                // No signature, display as unsigned message
                                let display_msg =
                                    format!("📝 {} says (unsigned): {}", message.from_name, message.content);
                                let _ = message_sender.send(display_msg);
                            }
                        }
                        NetworkMessage::Exit(peer_id) => {
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
                        NetworkMessage::Discovery(peer_info) => {
                            if peer_info.id == peer_id {
                                // Ignore our own Discovery messages
                                return Ok(());
                            }
                            // Validate discovered peer before adding
                            if !peer_info.is_valid() {
                                eprintln!("Invalid peer info received via TCP: {peer_info:?}");
                                return Ok(());
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
                        NetworkMessage::Heartbeat(_) => {}
                        NetworkMessage::SignedChat(signed_message) => {
                            if let Some(crypto_mgr) = &crypto_manager {
                                // Verify the signed message
                                match crypto_mgr.verify_message(&signed_message).await {
                                    Ok(true) => {
                                        let display_msg = format!(
                                            "🔐 {} says (verified): {}",
                                            signed_message.signer_name, signed_message.message
                                        );
                                        let _ = message_sender.send(display_msg);

                                        // Add the signer's public key to known keys
                                        if let Err(e) = crypto_mgr
                                            .add_known_peer(
                                                signed_message.signer_id.clone(),
                                                signed_message.public_key.clone(),
                                            )
                                            .await
                                        {
                                            eprintln!("Failed to add peer key: {e}");
                                        }
                                    }
                                    Ok(false) => {
                                        let display_msg = format!(
                                            "⚠️  {} says (INVALID SIGNATURE): {}",
                                            signed_message.signer_name, signed_message.message
                                        );
                                        let _ = message_sender.send(display_msg);
                                    }
                                    Err(e) => {
                                        let display_msg = format!(
                                            "❓ {} says (verification failed: {}): {}",
                                            signed_message.signer_name, e, signed_message.message
                                        );
                                        let _ = message_sender.send(display_msg);
                                    }
                                }
                            } else {
                                // No crypto manager, display as regular message
                                let display_msg = format!(
                                    "{} says: {}",
                                    signed_message.signer_name, signed_message.message
                                );
                                let _ = message_sender.send(display_msg);
                            }
                        }
                        NetworkMessage::IdentityAnnouncement {
                            peer_id,
                            name,
                            public_key,
                        } => {
                            if let Some(crypto_mgr) = &crypto_manager {
                                // Add the peer's public key to known keys
                                if let Err(e) = crypto_mgr
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
                        }
                        NetworkMessage::UpgradeRequest(proposal) => {
                            println!("🔐 Received upgrade proposal from {}: {}", 
                                proposal.proposer_name, proposal.description);
                            println!("📊 Proposal ID: {}, requires {}/{} approvals", 
                                proposal.proposal_id, proposal.required_approvals, proposal.total_peers);

                            // Store proposal locally if not present
                            threshold_manager.insert_received_proposal(proposal.clone()).await;

                            let display_msg = format!(
                                "🔐 {} proposed secure messaging upgrade: {} (ID: {})",
                                proposal.proposer_name, proposal.description, proposal.proposal_id
                            );
                            let _ = message_sender.send(display_msg);
                        }
                        NetworkMessage::UpgradeVote(vote) => {
                            println!("🗳️  Received vote from {} on proposal {}: {}", 
                                vote.voter_name, vote.proposal_id, 
                                if vote.approved { "✅ APPROVED" } else { "❌ REJECTED" });
                            
                            // TODO: Process vote locally
                            let _ = threshold_manager.handle_received_vote(
                                &vote
                            ).await;

                            let display_msg = format!(
                                "🗳️  {} voted {} on upgrade proposal {}",
                                vote.voter_name,
                                if vote.approved { "✅ APPROVED" } else { "❌ REJECTED" },
                                vote.proposal_id
                            );
                            let _ = message_sender.send(display_msg);
                        }
                        NetworkMessage::PartialSignature(partial_sig) => {
                            println!("🔐 Received partial signature from {} on proposal {}", 
                                partial_sig.signer_name, partial_sig.proposal_id);
                            
                            // TODO: Process partial signature for threshold verification
                            let display_msg = format!(
                                "🔐 {} provided partial signature for proposal {}",
                                partial_sig.signer_name, partial_sig.proposal_id
                            );
                            let _ = message_sender.send(display_msg);
                        }
                    }
                }
            }
            Err(e) => return Err(ChatError::Network(e.to_string())),
        }
    }
    Ok(())
}
