use crate::chat::Peer;
use std::sync::Arc;
use tokio::signal;

pub async fn handle_signals(wt: Arc<Peer>) {
    // Wait for either SIGINT (Ctrl+C) or SIGTERM
    let _ = signal::ctrl_c().await;
    // Call the quit procedure (same as /quit)
    println!("\u{1F44B} Now Goodbye!");
    wt.shutdown().await;
}
