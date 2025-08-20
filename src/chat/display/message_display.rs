//! Message display module: Responsible for displaying incoming messages to the user in the CLI.
//!
//! This module contains the `start_message_display` function, which listens for messages
//! on a broadcast channel and prints them to the standard output. It is designed to be
//! run asynchronously, and it expects a reference to a `Peer` instance, which
//! manages the underlying message sending and receiving.

use crate::chat::Peer;
use crate::error::ChatError;
use tokio::sync::broadcast;

pub async fn start_message_display(peer: &Peer) -> Result<(), ChatError> {
    let mut receiver = peer.message_sender.subscribe();
    loop {
        match receiver.recv().await {
            Ok(message) => {
                println!("\n📨 {}", message);
                print!("💬 ");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
            Err(broadcast::error::RecvError::Closed) => break,
            Err(broadcast::error::RecvError::Lagged(_)) => {
                eprintln!("Message display lagged, continuing...");
            }
        }
    }
    Ok(())
}
