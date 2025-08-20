use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "p2p_chat")]
#[command(about = "A simple peer-to-peer chat application")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Chat (discover peers and listen for messages)
    Start {
        /// Port to listen on for TCP connections
        #[arg(short, long, default_value = "9999")]
        port: u16,
        /// Your display name
        #[arg(short, long, default_value = "Anonymous")]
        name: String,
    },
}
