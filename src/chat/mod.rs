//! Chat module: Provides the main struct and logic for peer-to-peer Chat functionality, including peer management, message broadcasting, and coordination of submodules.
//!
//! This module defines the `Peer` struct, which represents a peer in the Chat network.
//! It handles the initialization of the peer, starting of necessary services like TCP listener,
//! mDNS discovery, heartbeat sending, and CLI handling. It also provides functionality to broadcast
//! messages to other peers.

pub mod net {
    pub mod broadcast;
    pub mod discovery;
    pub mod heartbeat;
    pub mod listener;
}

pub mod display {
    pub mod cli;
    pub mod message_display;
}

use crate::crypto::{threshold::ThresholdManager, CryptoManager};
use crate::error::ChatError;
use crate::peer::PeerInfo;
use colored::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct Peer {
    pub peer_id: String,
    pub name: String,
    pub port: u16,
    pub peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    pub message_sender: tokio::sync::broadcast::Sender<String>,
    pub crypto_manager: Arc<CryptoManager>,
    pub threshold_manager: Arc<ThresholdManager>,
}

impl Peer {
    pub fn new(name: String, port: u16) -> Self {
        // Validate name and port
        let valid_name = name.trim();
        let name = if valid_name.is_empty() || valid_name.len() > 128 {
            "Anonymous".to_string()
        } else {
            valid_name.to_string()
        };
        let port = if port == 0 { 8080 } else { port };
        let peer_id = Uuid::new_v4().to_string();
        let (message_sender, _) = tokio::sync::broadcast::channel(100);

        // Initialize cryptographic identity
        let crypto_manager = Arc::new(CryptoManager::new(peer_id.clone(), name.clone()));

        // Initialize threshold manager for secure-only messaging upgrades
        let threshold_manager = Arc::new(ThresholdManager::default());

        Self {
            peer_id,
            name,
            port,
            peers: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
            crypto_manager,
            threshold_manager,
        }
    }
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "🎙️  Starting P2P Chat...".bright_cyan().bold());
        println!("👤 Your ID: {}", self.peer_id.bright_yellow());
        println!("📡 Your Name: {}", self.name.bright_green());
        println!(
            "🔌 Listening on port: {}",
            self.port.to_string().bright_blue()
        );

        // Display cryptographic identity
        let identity = self.crypto_manager.get_identity();
        let public_key_hex = hex::encode(&identity.public_key);
        println!(
            "🔐 Your Public Key: {}",
            public_key_hex[..16].bright_magenta()
        );
        println!("🔐 Full Key: {}", public_key_hex.bright_magenta());

        // Start all services concurrently
        let tcp_listener = net::listener::start_tcp_listener(self);
        let mdns_discovery = net::discovery::start_mdns(Arc::new(self.clone()));
        let heartbeat_sender = net::heartbeat::start_heartbeat(self);

        // Create a single StdCliIO instance and pass a reference to the CLI handler so it
        // does not take a temporary reference to a temporary value.
        let cli_io = display::cli::StdCliIO;
        let cli_handler = display::cli::start_cli_handler(self);

        let message_display = display::message_display::start_message_display(self);

        tokio::select! {
            result = tcp_listener => {
                if let Err(e) = result {
                    eprintln!("TCP listener error: {e}");
                    self.shutdown().await;
                }
            }
            result = mdns_discovery => {
                if let Err(e) = result {
                    eprintln!("mDNS discovery error: {e}");
                    self.shutdown().await;
                }
            }
            result = heartbeat_sender => {
                if let Err(e) = result {
                    eprintln!("Heartbeat sender error: {e}");
                    self.shutdown().await;
                }
            }
            result = cli_handler => {
                if let Err(e) = result {
                    eprintln!("CLI handler error: {e}");
                    self.shutdown().await;
                }
            }
            result = message_display => {
                if let Err(e) = result {
                    eprintln!("Message display error: {e}");
                    self.shutdown().await;
                }
            }
        }
        Ok(())
    }
    pub async fn broadcast_message(&self, content: &str) -> Result<(), ChatError> {
        net::broadcast::broadcast_message(self, content).await
    }

    /// Broadcast a message without cryptographic signing
    pub async fn broadcast_unsigned_message(&self, content: &str) -> Result<(), ChatError> {
        net::broadcast::broadcast_unsigned_message(self, content).await
    }

    /// Create a proposal to enable secure-only messaging
    pub async fn propose_secure_upgrade(&self, description: &str) -> Result<String, ChatError> {
        let peers_count = self.peers.lock().await.len();
        let required_approvals = (peers_count / 2) + 1; // Simple majority rule

        let proposal_id = self
            .threshold_manager
            .create_proposal(
                self.peer_id.clone(),
                self.name.clone(),
                description.to_string(),
                required_approvals,
                peers_count + 1, // Include self
            )
            .await?;

        // Broadcast the proposal to all peers
        net::broadcast::broadcast_upgrade_proposal(self, &proposal_id).await?;

        println!("🔐 Created secure messaging upgrade proposal: {proposal_id}");
        println!(
            "📊 Requires {required_approvals}/{} approvals to enable",
            peers_count + 1
        );

        Ok(proposal_id)
    }

    /// Vote on an upgrade proposal
    pub async fn vote_on_proposal(
        &self,
        proposal_id: &str,
        approved: bool,
    ) -> Result<(), ChatError> {
        self.threshold_manager
            .cast_vote(
                proposal_id,
                self.peer_id.clone(),
                self.name.clone(),
                approved,
                &self.crypto_manager,
            )
            .await?;

        let vote_text = if approved {
            "✅ approved"
        } else {
            "❌ rejected"
        };
        println!("🗳️  {vote_text} upgrade proposal: {proposal_id}");

        // Broadcast the vote to all peers
        net::broadcast::broadcast_proposal_vote(self, proposal_id, approved).await?;

        Ok(())
    }

    /// Check if secure-only messaging is currently enabled
    pub async fn is_secure_only_enabled(&self) -> bool {
        self.threshold_manager.is_secure_only_enabled().await
    }

    /// Get active upgrade proposals
    pub async fn get_active_proposals(&self) -> Vec<crate::crypto::threshold::UpgradeProposal> {
        self.threshold_manager.get_active_proposals().await
    }

    /// Get votes for a specific proposal
    pub async fn get_proposal_votes(
        &self,
        proposal_id: &str,
    ) -> Vec<crate::crypto::threshold::UpgradeVote> {
        self.threshold_manager.get_proposal_votes(proposal_id).await
    }

    pub async fn shutdown(&self) {
        let _ = crate::chat::display::cli::broadcast_exit(self).await;

        // TODO: Wait for all network tasks to finish (e.g., join handles)
        // TODO: Close all open connections and resources
        // You may want to set a shutdown flag and notify background tasks
        println!("Peer is shutting down gracefully...");
        // Give some time for messages to flush
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_chat_new() {
        let peer = Peer::new("Tester".to_string(), 9000);
        assert_eq!(peer.name, "Tester");
        assert_eq!(peer.port, 9000);
    }
}
