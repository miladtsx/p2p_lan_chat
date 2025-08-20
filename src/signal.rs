use crate::chat::{display::cli::broadcast_exit, Peer};
use std::sync::Arc;
use tokio::signal;

pub async fn handle_signals(wt: Arc<Peer>) {
    // Wait for either SIGINT (Ctrl+C) or SIGTERM
    let _ = signal::ctrl_c().await;
    // Call the quit procedure (same as /quit)
    if let Err(e) = broadcast_exit(&wt).await {
        eprintln!("Error broadcasting exit: {}", e);
    }
    println!("\u{1F44B} Now Goodbye!");
    std::process::exit(0);
}
