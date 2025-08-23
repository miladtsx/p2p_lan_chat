//! Integration tests for threshold signature functionality
//! 
//! This module tests the complete lifecycle of secure-only messaging upgrades,
//! including proposal creation, voting, threshold verification, and enforcement.

use p2p_chat::chat::Peer;
use p2p_chat::crypto::threshold::{ThresholdManager, ProposalState};
use p2p_chat::crypto::CryptoManager;

#[tokio::test]
async fn test_complete_upgrade_lifecycle() {
    // Create a threshold manager
    let threshold_manager = ThresholdManager::new();
    
    // Create a crypto manager for signing
    let crypto_manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());
    
    // Test 1: Create a proposal
    let proposal_id = threshold_manager.create_proposal(
        "proposer".to_string(),
        "Proposer".to_string(),
        "Enable secure messaging".to_string(),
        2, // Requires 2 approvals
        3, // Total of 3 peers
    ).await.unwrap();
    
    assert!(!proposal_id.is_empty());
    
    // Test 2: Check initial state
    let state = threshold_manager.get_proposal_state(&proposal_id).await.unwrap();
    assert!(matches!(state, ProposalState::Open));
    
    let secure_enabled = threshold_manager.is_secure_only_enabled().await;
    assert!(!secure_enabled);
    
    // Test 3: First approval
    threshold_manager.cast_vote(
        &proposal_id,
        "voter1".to_string(),
        "Voter1".to_string(),
        true,
        &crypto_manager,
    ).await.unwrap();
    
    // Should still be open (threshold not met)
    let state = threshold_manager.get_proposal_state(&proposal_id).await.unwrap();
    assert!(matches!(state, ProposalState::Open));
    
    let secure_enabled = threshold_manager.is_secure_only_enabled().await;
    assert!(!secure_enabled);
    
    // Test 4: Second approval - should trigger threshold
    threshold_manager.cast_vote(
        &proposal_id,
        "voter2".to_string(),
        "Voter2".to_string(),
        true,
        &crypto_manager,
    ).await.unwrap();
    
    // Should now be approved
    let state = threshold_manager.get_proposal_state(&proposal_id).await.unwrap();
    assert!(matches!(state, ProposalState::Approved));
    
    let secure_enabled = threshold_manager.is_secure_only_enabled().await;
    assert!(secure_enabled);
    
    // Test 5: Verify vote counts
    let votes = threshold_manager.get_proposal_votes(&proposal_id).await;
    let approval_count = votes.iter().filter(|v| v.approved).count();
    let rejection_count = votes.iter().filter(|v| !v.approved).count();
    
    assert_eq!(approval_count, 2);
    assert_eq!(rejection_count, 0);
    assert_eq!(votes.len(), 2);
}

#[tokio::test]
async fn test_proposal_rejection() {
    let threshold_manager = ThresholdManager::new();
    let crypto_manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());
    
    // Create a proposal requiring 2 approvals from 3 peers
    let proposal_id = threshold_manager.create_proposal(
        "proposer".to_string(),
        "Proposer".to_string(),
        "Enable secure messaging".to_string(),
        2,
        3,
    ).await.unwrap();
    
    // First approval
    threshold_manager.cast_vote(
        &proposal_id,
        "voter1".to_string(),
        "Voter1".to_string(),
        true,
        &crypto_manager,
    ).await.unwrap();
    
    // First rejection
    threshold_manager.cast_vote(
        &proposal_id,
        "voter2".to_string(),
        "Voter2".to_string(),
        false,
        &crypto_manager,
    ).await.unwrap();
    
    // Should still be open (threshold not met)
    let state = threshold_manager.get_proposal_state(&proposal_id).await.unwrap();
    assert!(matches!(state, ProposalState::Open));
    
    let secure_enabled = threshold_manager.is_secure_only_enabled().await;
    assert!(!secure_enabled);
    
    // Verify vote counts
    let votes = threshold_manager.get_proposal_votes(&proposal_id).await;
    let approval_count = votes.iter().filter(|v| v.approved).count();
    let rejection_count = votes.iter().filter(|v| !v.approved).count();
    
    assert_eq!(approval_count, 1);
    assert_eq!(rejection_count, 1);
}

#[tokio::test]
async fn test_duplicate_voting_prevention() {
    let threshold_manager = ThresholdManager::new();
    let crypto_manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());
    
    let proposal_id = threshold_manager.create_proposal(
        "proposer".to_string(),
        "Proposer".to_string(),
        "Enable secure messaging".to_string(),
        1,
        2,
    ).await.unwrap();
    
    // First vote
    threshold_manager.cast_vote(
        &proposal_id,
        "voter1".to_string(),
        "Voter1".to_string(),
        true,
        &crypto_manager,
    ).await.unwrap();
    
    // Duplicate vote should fail
    let result = threshold_manager.cast_vote(
        &proposal_id,
        "voter1".to_string(),
        "Voter1".to_string(),
        false,
        &crypto_manager,
    ).await;
    
    assert!(result.is_err());
    
    // Verify only one vote exists
    let votes = threshold_manager.get_proposal_votes(&proposal_id).await;
    assert_eq!(votes.len(), 1);
}

#[tokio::test]
async fn test_multiple_proposals() {
    let threshold_manager = ThresholdManager::new();
    let crypto_manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());
    
    // Create first proposal
    let proposal1_id = threshold_manager.create_proposal(
        "proposer1".to_string(),
        "Proposer1".to_string(),
        "First upgrade proposal".to_string(),
        1,
        2,
    ).await.unwrap();
    
    // Create second proposal
    let proposal2_id = threshold_manager.create_proposal(
        "proposer2".to_string(),
        "Proposer2".to_string(),
        "Second upgrade proposal".to_string(),
        1,
        2,
    ).await.unwrap();
    
    // Both should be active
    let active_proposals = threshold_manager.get_active_proposals().await;
    assert_eq!(active_proposals.len(), 2);
    
    // Approve first proposal
    threshold_manager.cast_vote(
        &proposal1_id,
        "voter1".to_string(),
        "Voter1".to_string(),
        true,
        &crypto_manager,
    ).await.unwrap();
    
    // First should be approved, second still open
    let state1 = threshold_manager.get_proposal_state(&proposal1_id).await.unwrap();
    let state2 = threshold_manager.get_proposal_state(&proposal2_id).await.unwrap();
    
    assert!(matches!(state1, ProposalState::Approved));
    assert!(matches!(state2, ProposalState::Open));
    
    // Only one should be active now
    let active_proposals = threshold_manager.get_active_proposals().await;
    assert_eq!(active_proposals.len(), 1);
    assert_eq!(active_proposals[0].proposal_id, proposal2_id);
}

#[tokio::test]
async fn test_peer_integration() {
    // Create a peer with threshold manager
    let peer = Peer::new("TestPeer".to_string(), 9000);
    
    // Initially secure-only should be disabled
    assert!(!peer.is_secure_only_enabled().await);
    
    // Create a proposal
    let proposal_id = peer.propose_secure_upgrade("Test upgrade").await.unwrap();
    
    // Should have one active proposal
    let proposals = peer.get_active_proposals().await;
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].proposal_id, proposal_id);
    
    // Vote on the proposal
    peer.vote_on_proposal(&proposal_id, true).await.unwrap();
    
    // Since we're the only peer, this should trigger the threshold
    // (assuming the peer counts itself in the total)
    let secure_enabled = peer.is_secure_only_enabled().await;
    assert_eq!(secure_enabled, true);
}
