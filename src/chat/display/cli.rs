//! CLI display module: Handles user input, command parsing, and broadcasting messages or exit signals to peers.
//!
//! This module provides the functionality for the command-line interface (CLI) of the application,
//! allowing users to interact with the Chat network. It handles user commands such as
//! listing peers, sending messages, and quitting the application. Additionally, it manages the
//! broadcasting of exit signals to all connected peers when a user decides to quit.

use crate::chat::Peer;
use crate::error::ChatError;
use crate::peer::NetworkMessage;
use serde_json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use hex;

pub async fn broadcast_exit(peer: &Peer) -> Result<(), ChatError> {
    let exit_msg = NetworkMessage::Exit(peer.peer_id.clone());
    let msg_bytes = serde_json::to_vec(&exit_msg)?;
    let peers = peer.peers.lock().await;
    for peer in peers.values() {
        if let Ok(mut stream) = TcpStream::connect((peer.ip, peer.port)).await {
            let _ = stream.write_all(&msg_bytes).await;
            println!("Quit broadcasted to {} ({})", peer.name, peer.id);
        }
    }
    Ok(())
}

pub async fn start_cli_handler(peer: &Peer) -> Result<(), ChatError> {
    println!("\n📋 Commands:");
    println!("  /list    - List discovered peers");
    println!("  /msg <message> - Send signed message to all peers");
    println!("  /unsigned <message> - Send unsigned message to all peers");
    println!("  /crypto  - Show cryptographic information");
    println!("  /propose <description> - Propose secure-only messaging upgrade");
    println!("  /vote <proposal_id> <approve|reject> - Vote on upgrade proposal");
    println!("  /proposals - List active upgrade proposals");
    println!("  /status  - Show security status and proposals");
    println!("  /quit    - Quit the application");
    println!("  Just type any message to broadcast it (signed by default)!\n");

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        print!("💬 ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        // Validate input length
        if input.len() > 512 {
            println!("Input too long. Please keep messages under 512 characters.");
            continue;
        }

        let mut parts = input.splitn(2, ' ');
        let command = parts.next().unwrap();
        let args = parts.next().unwrap_or("");
        match command {
            "/quit" => {
                if let Err(e) = crate::chat::display::cli::broadcast_exit(peer).await {
                    eprintln!("Error broadcasting exit: {e}");
                }
                println!("\u{1F44B} Now Goodbye!");
                std::process::exit(0);
            }
            "/list" => {
                let peers = peer.peers.lock().await;
                if peers.is_empty() {
                    println!("📭 No peers discovered yet.");
                } else {
                    println!("👥 Discovered peers:");
                    for peer in peers.values() {
                        if !peer.is_valid() {
                            println!("  - Invalid peer: {peer:?}");
                            continue;
                        }
                        println!(
                            "  - {} ({}) at {}:{}",
                            peer.name, peer.id, peer.ip, peer.port
                        );
                    }
                }
            }
            "/crypto" => {
                let identity = peer.crypto_manager.get_identity();
                let public_key_hex = hex::encode(&identity.public_key);
                println!("🔐 Cryptographic Identity:");
                println!("  Peer ID: {}", identity.peer_id);
                println!("  Name: {}", identity.name);
                println!("  Public Key: {public_key_hex}");
                println!("  Known Peer Keys: {}", peer.crypto_manager.known_peers_count().await);
            }
            "/propose" => {
                let description = if args.is_empty() {
                    "Enable secure-only messaging for all future communications"
                } else {
                    args
                };
                
                match peer.propose_secure_upgrade(description).await {
                    Ok(proposal_id) => {
                        println!("✅ Upgrade proposal created successfully!");
                        println!("📋 Proposal ID: {}", proposal_id);
                    }
                    Err(e) => eprintln!("❌ Failed to create upgrade proposal: {e}"),
                }
            }
            "/vote" => {
                let parts: Vec<&str> = args.split_whitespace().collect();
                if parts.len() != 2 {
                    println!("❌ Usage: /vote <proposal_id> <approve|reject>");
                    continue;
                }
                
                let proposal_id = parts[0];
                let vote_str = parts[1].to_lowercase();
                
                let the_vote = match vote_str.as_str() {
                    "approve" | "yes" | "true" | "1" => true,
                    "reject" | "no" | "false" | "0" => false,
                    _ => {
                        println!("❌ Invalid vote. Use 'approve' or 'reject'");
                        continue;
                    }
                };
                
                match peer.vote_on_proposal(proposal_id, the_vote).await {
                    Ok(()) => {
                        let vote_text = if the_vote { "approved" } else { "rejected" };
                        println!("✅ Successfully {} upgrade proposal: {}", vote_text, proposal_id);
                    }
                    Err(e) => eprintln!("❌ Failed to vote on upgrade proposal: {e}"),
                }
            }
            "/proposals" => {
                let proposals = peer.get_active_proposals().await;
                if proposals.is_empty() {
                    println!("📭 No active upgrade proposals");
                } else {
                    println!("🔐 Active Upgrade Proposals:");
                    for proposal in proposals {
                        println!("  📋 ID: {}", proposal.proposal_id);
                        println!("    Proposed by: {} ({})", proposal.proposer_name, proposal.proposer_id);
                        println!("    Description: {}", proposal.description);
                        println!("    Required: {}/{} approvals", proposal.required_approvals, proposal.total_peers);
                        println!("    Created: {}", proposal.timestamp);
                        println!();
                    }
                }
            }
            "/status" => {
                let secure_enabled = peer.is_secure_only_enabled().await;
                let proposals = peer.get_active_proposals().await;
                
                println!("🔐 Security Status:");
                println!("  Secure-only messaging: {}", if secure_enabled { "✅ ENABLED" } else { "❌ DISABLED" });
                println!("  Active proposals: {}", proposals.len());
                
                if !proposals.is_empty() {
                    println!("\n📋 Active Proposals:");
                    for proposal in proposals {
                        let votes = peer.get_proposal_votes(&proposal.proposal_id).await;
                        let approval_count = votes.iter().filter(|v| v.approved).count();
                        let rejection_count = votes.iter().filter(|v| !v.approved).count();
                        
                        println!("  📋 {}: {}/{} approvals, {} rejections", 
                            proposal.proposal_id, approval_count, proposal.required_approvals, rejection_count);
                    }
                }
            }
            "/unsigned" => {
                let message_content = args;
                if let Err(e) = peer.broadcast_unsigned_message(message_content).await {
                    eprintln!("Failed to send unsigned message: {e}");
                }
            }
            _ => {
                let message_content = if input.starts_with("/msg ") {
                    input.strip_prefix("/msg ").unwrap()
                } else {
                    input
                };
                if let Err(e) = peer.broadcast_message(message_content).await {
                    eprintln!("Failed to send message: {e}");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_name_validation() {
        let p1 = Peer::new("".to_string(), 9000);
        assert_eq!(p1.name, "Anonymous");

        let invalid_name_length = 1000;
        let long_name = "a".repeat(invalid_name_length);
        let p2 = Peer::new(long_name, 9000);
        assert_eq!(p2.name, "Anonymous");

        let valid_name = "Bob".to_string();
        let p3 = Peer::new(valid_name.clone(), 9000);
        assert_eq!(p3.name, valid_name);
    }

    #[test]
    fn test_chat_port_validation() {
        let p1 = Peer::new("Alice".to_string(), 0);
        assert_eq!(p1.port, 8080);
        let p2 = Peer::new("Alice".to_string(), 1234);
        assert_eq!(p2.port, 1234);
    }
}
