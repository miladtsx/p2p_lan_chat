//! Handler functions to manage upgrade proposals and voting.


use crate::crypto::threshold::{PartialSignature, UpgradeProposal, UpgradeVote};
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn handle_upgrade_request(
    proposal: UpgradeProposal,
    threshold_manager: Arc<crate::crypto::threshold::ThresholdManager>,
    message_sender: &broadcast::Sender<String>,
) {
    println!(
        "🔐 Received upgrade proposal from {}: {}",
        proposal.proposer_name, proposal.description
    );
    println!(
        "📊 Proposal ID: {}, requires {}/{} approvals",
        proposal.proposal_id, proposal.required_approvals, proposal.total_peers
    );

    // Store proposal locally if not present
    threshold_manager
        .insert_received_proposal(proposal.clone())
        .await;

    let display_msg = format!(
        "🔐 {} proposed secure messaging upgrade: {} (ID: {})",
        proposal.proposer_name, proposal.description, proposal.proposal_id
    );
    let _ = message_sender.send(display_msg);
}

pub async fn handle_upgrade_vote(
    vote: UpgradeVote,
    threshold_manager: Arc<crate::crypto::threshold::ThresholdManager>,
    message_sender: &broadcast::Sender<String>,
) {
    println!(
        "🗳️  Received vote from {} on proposal {}: {}",
        vote.voter_name,
        vote.proposal_id,
        if vote.approved {
            "✅ APPROVED"
        } else {
            "❌ REJECTED"
        }
    );

    // TODO: Process vote locally
    let _ = threshold_manager.handle_received_vote(&vote).await;

    let display_msg = format!(
        "🗳️  {} voted {} on upgrade proposal {}",
        vote.voter_name,
        if vote.approved {
            "✅ APPROVED"
        } else {
            "❌ REJECTED"
        },
        vote.proposal_id
    );
    let _ = message_sender.send(display_msg);
}

pub async fn handle_partial_signature(
    partial_sig: PartialSignature,
    message_sender: &broadcast::Sender<String>,
) {
    println!(
        "🔐 Received partial signature from {} on proposal {}",
        partial_sig.signer_name, partial_sig.proposal_id
    );

    // TODO: Process partial signature for threshold verification
    let display_msg = format!(
        "🔐 {} provided partial signature for proposal {}",
        partial_sig.signer_name, partial_sig.proposal_id
    );
    let _ = message_sender.send(display_msg);
}
