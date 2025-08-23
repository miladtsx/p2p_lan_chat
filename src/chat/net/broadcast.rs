use crate::chat::Peer;
use crate::error::ChatError;
use crate::peer::{Message, NetworkMessage};
use serde_json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn broadcast_message(peer: &Peer, content: &str) -> Result<(), ChatError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ChatError::Unknown(e.to_string()))?
        .as_secs();

    // Create a signed message for cryptographic authenticity
    let signed_message = peer.crypto_manager.sign_message(content, timestamp)?;
    
    // Create both regular and signed message formats for compatibility
    let regular_message = Message {
        from_id: peer.peer_id.clone(),
        from_name: peer.name.clone(),
        content: content.to_string(),
        timestamp,
        signature: Some(signed_message.signature.clone()),
        public_key: Some(signed_message.public_key.clone()),
    };
    
    let signed_network_msg = NetworkMessage::SignedChat(signed_message);
    let regular_network_msg = NetworkMessage::Chat(regular_message);
    
    // Send both message types for maximum compatibility
    let signed_msg_bytes = serde_json::to_vec(&signed_network_msg)?;
    let regular_msg_bytes = serde_json::to_vec(&regular_network_msg)?;
    
    let peers = peer.peers.lock().await;
    let mut successful_sends = 0;
    
    for peer_info in peers.values() {
        if !peer_info.is_valid() {
            eprintln!("Skipping invalid peer: {peer_info:?}");
            continue;
        }
        
        if let Ok(mut stream) = TcpStream::connect((peer_info.ip, peer_info.port)).await {
            // Try to send signed message first, fallback to regular if needed
            let send_result = if stream.write_all(&signed_msg_bytes).await.is_ok() {
                Ok(())
            } else {
                stream.write_all(&regular_msg_bytes).await
            };
            
            if send_result.is_ok() {
                successful_sends += 1;
            }
        }
    }
    
    if successful_sends > 0 {
        println!("📤 Signed message sent to {successful_sends} peer(s)");
        println!("🔐 Message signed with Ed25519 for authenticity");
        println!("📊 Message details: content='{content}', timestamp={timestamp}");
    } else {
        println!("📭 No peers available to receive the message");
    }
    Ok(())
}

/// Broadcast the peer's identity with public key to all known peers
pub async fn broadcast_identity(peer: &Peer) -> Result<(), ChatError> {
    let identity = peer.crypto_manager.get_identity();
    let network_msg = NetworkMessage::IdentityAnnouncement {
        peer_id: identity.peer_id.clone(),
        name: identity.name.clone(),
        public_key: identity.public_key.clone(),
    };
    
    let msg_bytes = serde_json::to_vec(&network_msg)?;
    let peers = peer.peers.lock().await;
    let mut successful_sends = 0;
    
    for peer_info in peers.values() {
        if !peer_info.is_valid() {
            continue;
        }
        
        if let Ok(mut stream) = TcpStream::connect((peer_info.ip, peer_info.port)).await {
            if stream.write_all(&msg_bytes).await.is_ok() {
                successful_sends += 1;
            }
        }
    }
    
    if successful_sends > 0 {
        println!("🔐 Identity announced to {successful_sends} peer(s)");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::peer::PeerInfo;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_peerinfo_is_valid_for_broadcast() {
        let valid_peer = PeerInfo {
            id: "id1".to_string(),
            name: "Peer1".to_string(),
            ip: IpAddr::from_str("192.168.1.10").unwrap(),
            port: 9000,
        };
        assert!(valid_peer.is_valid());

        let invalid_peer = PeerInfo {
            id: "".to_string(),
            name: "".to_string(),
            ip: IpAddr::from_str("0.0.0.0").unwrap(),
            port: 0,
        };
        assert!(!invalid_peer.is_valid());
    }
}
