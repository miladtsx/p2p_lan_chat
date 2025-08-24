//! Command module: Defines traits and functions for network commands.

use crate::error::ChatError;
use crate::network::handlers;
use crate::peer::{NetworkMessage, PeerInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[async_trait]
pub trait NetworkCommand: Send {
    async fn execute(
        self: Box<Self>,
        peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
        message_sender: broadcast::Sender<String>,
        peer_id: String,
        threshold_manager: Arc<crate::crypto::threshold::ThresholdManager>,
        crypto_manager: Arc<crate::crypto::CryptoManager>,
    ) -> Result<(), ChatError>;
}

#[async_trait]
impl NetworkCommand for NetworkMessage {
    async fn execute(
        self: Box<Self>,
        peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
        message_sender: broadcast::Sender<String>,
        peer_id: String,
        threshold_manager: Arc<crate::crypto::threshold::ThresholdManager>,
        crypto_manager: Arc<crate::crypto::CryptoManager>,
    ) -> Result<(), ChatError> {
        match *self {
            NetworkMessage::Chat(message) => {
                handlers::chat::handle_chat_message(message, &message_sender, &crypto_manager)
                    .await;
                Ok(())
            }
            NetworkMessage::Exit(peer_id) => {
                handlers::peer::handle_exit(&peers, peer_id).await;
                Ok(())
            }
            NetworkMessage::Discovery(peer_info) => {
                handlers::peer::handle_discovery(&peers, peer_info, peer_id.clone()).await;
                Ok(())
            }
            NetworkMessage::Heartbeat(_) => {
                handlers::peer::handle_heartbeat().await;
                Ok(())
            }
            NetworkMessage::SignedChat(signed_message) => {
                handlers::chat::handle_signed_chat(
                    signed_message,
                    &message_sender,
                    &crypto_manager,
                )
                .await;
                Ok(())
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
                .await;
                Ok(())
            }
            NetworkMessage::UpgradeRequest(proposal) => {
                handlers::upgrade::handle_upgrade_request(
                    proposal,
                    threshold_manager.clone(),
                    &message_sender,
                )
                .await;
                Ok(())
            }
            NetworkMessage::UpgradeVote(vote) => {
                handlers::upgrade::handle_upgrade_vote(
                    vote,
                    threshold_manager.clone(),
                    &message_sender,
                )
                .await;
                Ok(())
            }
            NetworkMessage::PartialSignature(partial_sig) => {
                handlers::upgrade::handle_partial_signature(partial_sig, &message_sender).await;
                Ok(())
            }
        }
    }
}

/// Factory to convert NetworkMessage to NetworkCommand
pub fn to_command(msg: NetworkMessage) -> Box<dyn NetworkCommand + Send> {
    Box::new(msg)
}
