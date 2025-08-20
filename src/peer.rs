//! Peer module: Defines peer information, message types, and network message enums for the P2P Chat.
//!
//! This module contains the structures and enums used for peer discovery, messaging, and network
//! communication in the P2P Chat application. It includes the `PeerInfo` struct for
//! identifying peers in the network, the `Message` struct for chat messages, and the `NetworkMessage`
//! enum for different types of network messages.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
}

impl PeerInfo {
    /// Validate the fields of a PeerInfo instance.
    pub fn is_valid(&self) -> bool {
        !self.id.trim().is_empty()
            && !self.name.trim().is_empty()
            && self.name.len() <= 128
            && self.port > 0
            && !self.ip.is_loopback()
            && !self.ip.is_multicast()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from_id: String,
    pub from_name: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Discovery(PeerInfo),
    Chat(Message),
    Heartbeat(String), // peer_id
    Exit(String),      // peer_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_peer_info_valid() {
        let valid_peer = PeerInfo {
            id: "abc123".to_string(),
            name: "Alice".to_string(),
            ip: IpAddr::from_str("192.168.1.2").unwrap(),
            port: 9000,
        };
        assert!(valid_peer.is_valid());

        let invalid_peer = PeerInfo {
            id: "".to_string(),
            name: "".to_string(),
            ip: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 0,
        };
        assert!(!invalid_peer.is_valid());
    }

    #[test]
    fn test_peer_name_length() {
        let long_name = "a".repeat(1000);
        let p1 = PeerInfo {
            id: "id".to_string(),
            name: long_name,
            ip: IpAddr::from_str("10.0.0.1").unwrap(),
            port: 1234,
        };
        assert!(!p1.is_valid());
    }

    #[test]
    fn test_message_content() {
        let msg = Message {
            from_id: "id1".to_string(),
            from_name: "Alice".to_string(),
            content: "Hello, world!".to_string(),
            timestamp: 1234567890,
        };
        assert_eq!(msg.content, "Hello, world!");
        assert!(!msg.content.is_empty());
    }

    #[test]
    fn test_message_empty_content() {
        let msg = Message {
            from_id: "id2".to_string(),
            from_name: "Bob".to_string(),
            content: "".to_string(),
            timestamp: 1234567890,
        };
        assert!(msg.content.is_empty());
    }
}
