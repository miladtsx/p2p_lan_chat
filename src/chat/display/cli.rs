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
    println!("  /msg <message> - Send message to all peers");
    println!("  /quit    - Quit the application");
    println!("  Just type any message to broadcast it!\n");

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
        match input {
            "/quit" => {
                if let Err(e) = crate::chat::display::cli::broadcast_exit(peer).await {
                    eprintln!("Error broadcasting exit: {}", e);
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
                            println!("  - Invalid peer: {:?}", peer);
                            continue;
                        }
                        println!(
                            "  - {} ({}) at {}:{}",
                            peer.name, peer.id, peer.ip, peer.port
                        );
                    }
                }
            }
            _ => {
                let message_content = if input.starts_with("/msg ") {
                    input.strip_prefix("/msg ").unwrap()
                } else {
                    input
                };
                if let Err(e) = peer.broadcast_message(message_content).await {
                    eprintln!("Failed to send message: {}", e);
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
