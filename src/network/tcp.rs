//! TCP network module: Handles TCP connection logic for peer communication in the P2P Chat.
//!
//! This module is responsible for managing TCP connections with peers,
//! handling incoming messages, and broadcasting outgoing messages.
//! It utilizes Tokio's asynchronous runtime for non-blocking I/O operations.

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
) -> Result<(), ChatError> {
    let mut buf = [0; 1024];

    while let Ok(_n) = stream.readable().await {
        match stream.try_read(&mut buf) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                if let Ok(network_msg) = serde_json::from_slice::<NetworkMessage>(&buf[..n]) {
                    match network_msg {
                        NetworkMessage::Chat(message) => {
                            let display_msg =
                                format!("{} says: {}", message.from_name, message.content);
                            let _ = message_sender.send(display_msg);
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
                                eprintln!("Invalid peer info received via TCP: {:?}", peer_info);
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
                    }
                }
            }
            Err(e) => return Err(ChatError::Network(e.to_string())),
        }
    }
    Ok(())
}
