//! TCP network module: Handles TCP connection logic for peer communication in the P2P Chat.
//!
//! This module is responsible for managing TCP connections with peers,
//! handling incoming messages, and broadcasting outgoing messages.
//! It utilizes Tokio's asynchronous runtime for non-blocking I/O operations.

use crate::error::ChatError;
use crate::network::command::to_command;
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
                    let command = to_command(network_msg);
                    command
                        .execute(
                            peers.clone(),
                            message_sender.clone(),
                            peer_id.clone(),
                            threshold_manager.clone(),
                            crypto_manager.clone(),
                        )
                        .await?;
                }
            }
            Err(e) => return Err(ChatError::Network(e.to_string())),
        }
    }
    Ok(())
}
