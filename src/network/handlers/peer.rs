//! Peer helper functions to handle peer functionality such as discovery, identity management, and connection handling.

use crate::peer::PeerInfo;
use chrono::Utc;
use colored::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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
