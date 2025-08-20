use crate::chat::Peer;
use crate::error::ChatError;
use crate::peer::{Message, NetworkMessage};
use serde_json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn broadcast_message(peer: &Peer, content: &str) -> Result<(), ChatError> {
    let message = Message {
        from_id: peer.peer_id.clone(),
        from_name: peer.name.clone(),
        content: content.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ChatError::Unknown(e.to_string()))?
            .as_secs(),
    };
    let network_msg = NetworkMessage::Chat(message.clone());
    let msg_bytes = serde_json::to_vec(&network_msg)?;
    let peers = peer.peers.lock().await;
    let mut successful_sends = 0;
    for peer in peers.values() {
        if !peer.is_valid() {
            eprintln!("Skipping invalid peer: {:?}", peer);
            continue;
        }
        if let Ok(mut stream) = TcpStream::connect((peer.ip, peer.port)).await {
            if stream.write_all(&msg_bytes).await.is_ok() {
                successful_sends += 1;
            }
        }
    }
    if successful_sends > 0 {
        println!("📤 Message sent to {} peer(s)", successful_sends);
    } else {
        println!("📭 No peers available to receive the message");
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
