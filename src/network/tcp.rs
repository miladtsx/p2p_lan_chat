//! TCP network module: Handles TCP connection logic for peer communication in the P2P Chat.
//!
//! This module is responsible for managing TCP connections with peers,
//! handling incoming messages, and broadcasting outgoing messages.
//! It utilizes Tokio's asynchronous runtime for non-blocking I/O operations.

use crate::error::ChatError;
use crate::network::handlers;
use crate::peer::{NetworkMessage, PeerInfo};
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
                            handlers::chat::handle_chat_message(
                                message,
                                &message_sender,
                                &crypto_manager,
                            )
                            .await
                        }
                        NetworkMessage::Exit(peer_id) => {
                            handlers::peer::handle_exit(&peers, peer_id).await
                        }
                        NetworkMessage::Discovery(peer_info) => {
                            handlers::peer::handle_discovery(&peers, peer_info, peer_id.clone())
                                .await
                        }
                        NetworkMessage::Heartbeat(_) => {
                            handlers::peer::handle_heartbeat().await;
                        }
                        NetworkMessage::SignedChat(signed_message) => {
                            handlers::chat::handle_signed_chat(
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
                            handlers::peer::handle_identity_announcement(
                                peer_id,
                                name,
                                public_key,
                                &crypto_manager,
                            )
                            .await
                        }
                        NetworkMessage::UpgradeRequest(proposal) => {
                            handlers::upgrade::handle_upgrade_request(
                                proposal,
                                threshold_manager.clone(),
                                &message_sender,
                            )
                            .await
                        }
                        NetworkMessage::UpgradeVote(vote) => {
                            handlers::upgrade::handle_upgrade_vote(
                                vote,
                                threshold_manager.clone(),
                                &message_sender,
                            )
                            .await
                        }
                        NetworkMessage::PartialSignature(partial_sig) => {
                            handlers::upgrade::handle_partial_signature(
                                partial_sig,
                                &message_sender,
                            )
                            .await
                        }
                    }
                }
            }
            Err(e) => return Err(ChatError::Network(e.to_string())),
        }
    }
    Ok(())
}
