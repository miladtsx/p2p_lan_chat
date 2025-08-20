//! TCP listener module: Listens for incoming TCP connections from peers and delegates connection handling.
//!
//! This module is responsible for starting a TCP listener on a specified port,
//! accepting incoming TCP connections, and spawning a new task to handle each
//! connection. It utilizes the `handle_tcp_connection` function from the
//! `network::tcp` module to process the connections.

use crate::chat::Peer;
use crate::network::tcp::handle_tcp_connection;
use colored::*;
use tokio::net::TcpListener;

pub async fn start_tcp_listener(peer: &Peer) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", peer.port);
    let listener = TcpListener::bind(&addr).await?;
    println!(
        "🔗 TCP listener started on port {}",
        peer.port.to_string().bright_blue()
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        let peers = peer.peers.clone();
        let message_sender = peer.message_sender.clone();
        let peer_id = peer.peer_id.clone();

        tokio::spawn(async move {
            if let Err(e) =
                handle_tcp_connection(stream, addr, peers, message_sender, peer_id).await
            {
                eprintln!("Error handling TCP connection from {}: {}", addr, e);
            }
        });
    }
}
