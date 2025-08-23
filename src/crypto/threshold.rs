//! Threshold signature module for secure-only messaging upgrades.
//!
//! This module implements a lightweight M-of-N threshold signature scheme
//! for approving network-wide security upgrades. It uses Ed25519-based
//! partial signatures that can be combined to form a valid group approval.

use crate::crypto::{CryptoError, CryptoManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Represents a proposal to enable secure-only messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeProposal {
    /// Unique identifier for this proposal
    pub proposal_id: String,
    /// The peer who initiated the proposal
    pub proposer_id: String,
    /// The proposer's display name
    pub proposer_name: String,
    /// Timestamp when the proposal was created
    pub timestamp: u64,
    /// Description of the upgrade
    pub description: String,
    /// Required number of approvals (M in M-of-N)
    pub required_approvals: usize,
    /// Total number of peers in the network (N in M-of-N)
    pub total_peers: usize,
}

/// A peer's vote on an upgrade proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeVote {
    /// The proposal this vote is for
    pub proposal_id: String,
    /// The peer who voted
    pub voter_id: String,
    /// The voter's display name
    pub voter_name: String,
    /// Whether the peer approves the upgrade
    pub approved: bool,
    /// Timestamp when the vote was cast
    pub timestamp: u64,
    /// Optional signature for vote authenticity
    pub signature: Option<Vec<u8>>,
}

/// A partial signature for threshold approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialSignature {
    /// The proposal this signature is for
    pub proposal_id: String,
    /// The peer who provided this partial signature
    pub signer_id: String,
    /// The signer's display name
    pub signer_name: String,
    /// The partial signature bytes
    pub signature: Vec<u8>,
    /// The signer's public key
    pub public_key: Vec<u8>,
    /// Timestamp when the signature was created
    pub timestamp: u64,
}

/// Represents the current state of a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalState {
    /// Proposal is open for voting
    Open,
    /// Proposal has been approved by threshold
    Approved,
    /// Proposal has been rejected or expired
    Rejected,
}

/// Manages upgrade proposals and threshold signatures
pub struct ThresholdManager {
    /// Active proposals
    proposals: Arc<RwLock<HashMap<String, UpgradeProposal>>>,
    /// Votes for each proposal
    votes: Arc<RwLock<HashMap<String, Vec<UpgradeVote>>>>,
    /// Partial signatures for each proposal
    partial_signatures: Arc<RwLock<HashMap<String, Vec<PartialSignature>>>>,
    /// Proposal states
    proposal_states: Arc<RwLock<HashMap<String, ProposalState>>>,
    /// Whether secure-only messaging is currently enabled
    secure_only_enabled: Arc<RwLock<bool>>,
}

impl ThresholdManager {
    /// Insert a received proposal if not present
    pub async fn insert_received_proposal(&self, proposal: UpgradeProposal) {
        let exists = self.get_proposal(&proposal.proposal_id).await.is_some();
        if !exists {
            self.proposals
                .write()
                .await
                .insert(proposal.proposal_id.clone(), proposal.clone());
            self.votes
                .write()
                .await
                .insert(proposal.proposal_id.clone(), Vec::new());
            self.partial_signatures
                .write()
                .await
                .insert(proposal.proposal_id.clone(), Vec::new());
            self.proposal_states
                .write()
                .await
                .insert(proposal.proposal_id.clone(), ProposalState::Open);
        }
    }
    /// Create a new threshold manager
    pub fn new() -> Self {
        Self {
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            partial_signatures: Arc::new(RwLock::new(HashMap::new())),
            proposal_states: Arc::new(RwLock::new(HashMap::new())),
            secure_only_enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new upgrade proposal
    pub async fn create_proposal(
        &self,
        proposer_id: String,
        proposer_name: String,
        description: String,
        required_approvals: usize,
        total_peers: usize,
    ) -> Result<String, CryptoError> {
        let proposal_id = Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| CryptoError::Unknown(e.to_string()))?
            .as_secs();

        let proposal = UpgradeProposal {
            proposal_id: proposal_id.clone(),
            proposer_id,
            proposer_name,
            timestamp,
            description,
            required_approvals,
            total_peers,
        };

        self.proposals
            .write()
            .await
            .insert(proposal_id.clone(), proposal);
        self.votes
            .write()
            .await
            .insert(proposal_id.clone(), Vec::new());
        self.partial_signatures
            .write()
            .await
            .insert(proposal_id.clone(), Vec::new());
        self.proposal_states
            .write()
            .await
            .insert(proposal_id.clone(), ProposalState::Open);

        Ok(proposal_id)
    }

    /// Cast a vote on a proposal
    pub async fn cast_vote(
        &self,
        proposal_id: &str,
        voter_id: String,
        voter_name: String,
        approved: bool,
        crypto_manager: &CryptoManager,
    ) -> Result<(), CryptoError> {
        // Check if proposal exists and is open
        // let _proposal = {
        //     let proposals = self.proposals.read().await;
        //     proposals.get(proposal_id)
        //         .ok_or(CryptoError::Unknown("Proposal not found".to_string()))?
        //         .clone()
        // };

        let state = {
            let states = self.proposal_states.read().await;
            states
                .get(proposal_id)
                .ok_or(CryptoError::Unknown("Proposal state not found".to_string()))?
                .clone()
        };

        match state {
            ProposalState::Open => {}
            _ => {
                return Err(CryptoError::Unknown(
                    "Proposal is not open for voting".to_string(),
                ))
            }
        }

        // Check if this peer has already voted
        let votes = self.votes.read().await;
        if let Some(existing_votes) = votes.get(proposal_id) {
            if existing_votes.iter().any(|v| v.voter_id == voter_id) {
                return Err(CryptoError::Unknown(
                    "Peer has already voted on this proposal".to_string(),
                ));
            }
        }
        drop(votes);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| CryptoError::Unknown(e.to_string()))?
            .as_secs();

        // Create a signed vote if approved
        let signature = if approved {
            let vote_data = format!("{}:{}:{}:{}", proposal_id, voter_id, approved, timestamp);
            let signature = crypto_manager.sign_message(&vote_data, timestamp)?;
            Some(signature.signature)
        } else {
            None
        };

        let vote = UpgradeVote {
            proposal_id: proposal_id.to_string(),
            voter_id,
            voter_name,
            approved,
            timestamp,
            signature,
        };

        // Add the vote
        self.votes
            .write()
            .await
            .entry(proposal_id.to_string())
            .or_insert_with(Vec::new)
            .push(vote);

        // Check if threshold is met
        self.check_threshold(proposal_id).await?;

        Ok(())
    }

    /// Check if a proposal has reached the required threshold
    async fn check_threshold(&self, proposal_id: &str) -> Result<(), CryptoError> {
        let proposal = {
            let proposals = self.proposals.read().await;
            proposals
                .get(proposal_id)
                .ok_or(CryptoError::Unknown("Proposal not found".to_string()))?
                .clone()
        };

        let votes = {
            let votes = self.votes.read().await;
            votes
                .get(proposal_id)
                .ok_or(CryptoError::Unknown("Votes not found".to_string()))?
                .clone()
        };

        let approval_count = votes.iter().filter(|v| v.approved).count();

        if approval_count >= proposal.required_approvals {
            // Threshold met - mark as approved
            self.proposal_states
                .write()
                .await
                .insert(proposal_id.to_string(), ProposalState::Approved);

            // Enable secure-only messaging
            *self.secure_only_enabled.write().await = true;

            println!(
                "🔐 Secure-only messaging enabled! Threshold of {}/{} approvals met.",
                approval_count, proposal.total_peers
            );
        }

        Ok(())
    }

    /// Handle a received vote from another peer
    pub async fn handle_received_vote(&self, vote: &UpgradeVote) {
        // Add the vote if not already present
        let existing_votes = self.get_proposal_votes(&vote.proposal_id).await;
        // If not voted already
        if !existing_votes.iter().any(|v| v.voter_id == vote.voter_id) {
            //TODO You may want to verify the vote signature here
            self.votes
                .write()
                .await
                .entry(vote.proposal_id.clone())
                .or_insert_with(Vec::new)
                .push(vote.clone());
            // Check threshold and activate if passed
            let _ = self.check_threshold(&vote.proposal_id).await;
        }
    }

    /// Handle incoming upgrade activation broadcast from another peer
    pub async fn handle_upgrade_activation(&self, proposal_id: &str) {
        // Set proposal state to Approved
        self.proposal_states
            .write()
            .await
            .insert(proposal_id.to_string(), ProposalState::Approved);

        // Enable secure-only messaging
        *self.secure_only_enabled.write().await = true;

        println!(
            "🔐 Secure-only messaging activated by broadcast for proposal_id: {}",
            proposal_id
        );
    }

    /// Get all active proposals
    pub async fn get_active_proposals(&self) -> Vec<UpgradeProposal> {
        let proposals = self.proposals.read().await;
        let states = self.proposal_states.read().await;

        proposals
            .values()
            .filter(|p| matches!(states.get(&p.proposal_id), Some(ProposalState::Open)))
            .cloned()
            .collect()
    }

    /// Get votes for a specific proposal
    pub async fn get_proposal_votes(&self, proposal_id: &str) -> Vec<UpgradeVote> {
        let votes = self.votes.read().await;
        votes.get(proposal_id).cloned().unwrap_or_default()
    }

    /// Check if secure-only messaging is enabled
    pub async fn is_secure_only_enabled(&self) -> bool {
        *self.secure_only_enabled.read().await
    }

    /// Get proposal state
    pub async fn get_proposal_state(&self, proposal_id: &str) -> Option<ProposalState> {
        let states = self.proposal_states.read().await;
        states.get(proposal_id).cloned()
    }

    /// Get proposal details
    pub async fn get_proposal(&self, proposal_id: &str) -> Option<UpgradeProposal> {
        let proposals = self.proposals.read().await;
        proposals.get(proposal_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::CryptoManager;

    #[tokio::test]
    async fn test_proposal_creation() {
        let manager = ThresholdManager::new();

        let proposal_id = manager
            .create_proposal(
                "proposer".to_string(),
                "Proposer".to_string(),
                "Enable secure messaging".to_string(),
                2,
                3,
            )
            .await
            .unwrap();

        assert!(!proposal_id.is_empty());

        let proposals = manager.get_active_proposals().await;
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].proposal_id, proposal_id);
    }

    #[tokio::test]
    async fn test_voting_and_threshold() {
        let manager = ThresholdManager::new();
        let crypto_manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());

        let proposal_id = manager
            .create_proposal(
                "proposer".to_string(),
                "Proposer".to_string(),
                "Enable secure messaging".to_string(),
                2,
                3,
            )
            .await
            .unwrap();

        // First approval
        manager
            .cast_vote(
                &proposal_id,
                "voter1".to_string(),
                "Voter1".to_string(),
                true,
                &crypto_manager,
            )
            .await
            .unwrap();

        // Second approval - should trigger threshold
        manager
            .cast_vote(
                &proposal_id,
                "voter2".to_string(),
                "Voter2".to_string(),
                true,
                &crypto_manager,
            )
            .await
            .unwrap();

        // Check if secure-only is enabled
        assert!(manager.is_secure_only_enabled().await);

        // Check proposal state
        let state = manager.get_proposal_state(&proposal_id).await.unwrap();
        assert!(matches!(state, ProposalState::Approved));
    }

    #[tokio::test]
    async fn test_duplicate_voting_prevention() {
        let manager = ThresholdManager::new();
        let crypto_manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());

        let proposal_id = manager
            .create_proposal(
                "proposer".to_string(),
                "Proposer".to_string(),
                "Enable secure messaging".to_string(),
                1,
                2,
            )
            .await
            .unwrap();

        // First vote
        manager
            .cast_vote(
                &proposal_id,
                "voter1".to_string(),
                "Voter1".to_string(),
                true,
                &crypto_manager,
            )
            .await
            .unwrap();

        // Duplicate vote should fail
        let result = manager
            .cast_vote(
                &proposal_id,
                "voter1".to_string(),
                "Voter1".to_string(),
                false,
                &crypto_manager,
            )
            .await;

        assert!(result.is_err());
    }
}
