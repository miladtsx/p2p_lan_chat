//! Main entry point: Sets up CLI, initializes the Chat, and starts the async runtime.
//!
//! This module is responsible for parsing command line arguments using Clap,
//! and starting the Chat service which facilitates peer-to-peer
//! communication over a network.

use std::sync::Arc;
use p2p_chat::chat::Peer;
use p2p_chat::cli::*;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    // Only handle CLI commands
    match cli.command {
        Commands::Start { port, name } => {
            let chat = Peer::new(name, port);
            let chat_arc = Arc::new(chat);
            let chat_signal = chat_arc.clone();
            tokio::spawn(async move {
                p2p_chat::signal::handle_signals(chat_signal).await;
            });
            chat_arc.start().await?;
        }
    }

    Ok(())
}
