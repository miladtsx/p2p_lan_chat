# Building a Peer-to-Peer Chat App: From Broadcast to mDNS

## Executive Summary

This post is a personal technical log of my journey building a peer-to-peer (P2P) chat application in Rust, focusing on the migration from UDP broadcast-based peer discovery to mDNS (Multicast DNS). It captures my learning process, the architectural decisions I made, the Rust-specific challenges I ran into, and the real-world debugging I had to do along the way.

---

## Why Not libp2p? Why libmdns and mdns?

One thing I want to clarify early: I did not use `libp2p` for this project. Instead, I chose to work directly with the `libmdns` and `mdns` crates. Why?

- **Simplicity and Focus:** `libp2p` is a powerful, modular framework for building complex, production-grade P2P networks. But for a small LAN chat, it felt like overkill. I wanted to keep things lightweight and have full control over the peer discovery process.
- **Learning Opportunity:** By using `libmdns` and `mdns` directly, I could get hands-on experience with how mDNS works under the hood, rather than relying on a higher-level abstraction. This was important for my own learning and curiosity.
- **Project Scope:** My goal was to build a simple, local network chat—not a full-featured, extensible P2P stack. I didn’t need NAT traversal, encryption, or multiplexing—just local peer discovery and messaging.
- **Debuggability:** With fewer abstractions, it was easier to see what was happening on the wire and debug network issues as they came up.

If you’re building something bigger or need advanced features, `libp2p` is a great choice. But for this project, direct mDNS was the right fit for my goals and learning style.

---

## Introduction

Peer-to-peer networking is a core building block for decentralized applications, but I had never built something like this before. In this post, I’m documenting my experience as I learned how to build a P2P chat app in Rust, and how I moved from a simple UDP broadcast mechanism to a more robust mDNS-based discovery protocol. This is not a tutorial or a guide—just a record of what I tried, what worked, what didn’t, and what I learned.

---

## Visual Overview

Below is a high-level diagram contrasting the two peer discovery approaches I experimented with:

```
+-------------------+         +-------------------+
|   UDP Broadcast   |         |      mDNS         |
+-------------------+         +-------------------+
| 1. Peer sends     |         | 1. Peer advertises|
|    broadcast      |         |    service via    |
|    packet         |         |    mDNS           |
| 2. All peers      |         | 2. Peers listen   |
|    receive and    |         |    for mDNS       |
|    respond        |         |    announcements  |
+-------------------+         +-------------------+
```

---

## Initial Architecture: UDP Broadcast Discovery

The first version of the chat app used UDP broadcast for peer discovery. Each node periodically sent a broadcast packet to a predefined port. All listening peers would respond, allowing everyone to learn about each other’s presence.

**Key points:**

- Simple to implement: just send and receive UDP packets.
- Worked well on local networks with minimal configuration.
- Peers identified by their IP and port.

**Limitations:**

- Broadcast traffic doesn’t cross subnets or VLANs.
- Many networks block broadcast for security reasons.
- Not scalable: as the number of peers grows, so does the broadcast traffic.
- No support for richer peer identity or service metadata.

---

## Why mDNS?

mDNS is designed for service discovery on local networks. It allows devices to find each other without a central server, using multicast DNS queries and responses. It’s widely used in consumer devices (e.g., AirPlay, Chromecast).

**Advantages:**

- No central server required.
- Works across most home and office LANs.
- Supports richer service metadata (service names, TXT records).
- More robust and scalable than broadcast.

---

## Migrating to mDNS: Technical Steps

### 1. Choosing a Rust mDNS Library

After evaluating options, I chose the [`libp2p-mdns`](https://docs.rs/libp2p/latest/libp2p/mdns/index.html) crate, which integrates with the broader `libp2p` ecosystem. This allowed for future extensibility (e.g., supporting other discovery protocols or transports). The Rust async ecosystem, especially with `tokio`, made it possible to handle network events efficiently.

### 2. Refactoring Peer Discovery

The original codebase had peer discovery logic tightly coupled to UDP broadcast. To support mDNS, I:

- Abstracted peer discovery into a trait (`PeerDiscovery`), with implementations for both broadcast and mDNS.
- Updated the main event loop to handle asynchronous mDNS events (using Rust’s async/await and `tokio` runtime).
- Ensured the chat protocol could work with both mechanisms for easier testing and fallback.

**Before (UDP Broadcast):**

```rust
// Pseudocode
fn broadcast_discover() {
    send_udp_broadcast();
    for packet in receive_udp() {
        add_peer(packet.source_ip);
    }
}
```

**After (mDNS):**

```rust
// Pseudocode
async fn mdns_discover() {
    let mdns = Mdns::new(MdnsConfig::default()).await?;
    while let Some(event) = mdns.next().await {
        match event {
            MdnsEvent::Discovered(peers) => {
                for (peer_id, _addr) in peers {
                    add_peer(peer_id);
                }
            }
            _ => {}
        }
    }
}
```

### 3. Handling mDNS Events

mDNS discovery is event-driven. Peers announce their presence and listen for others. The app needed to:

- Register a unique service name (e.g., `_p2p-chat._udp.local`).
- Listen for new peer announcements and update the peer list.
- Handle peer disappearance (e.g., timeouts or explicit "goodbye" messages).

### 4. Peer Identity and Metadata

With mDNS, each peer can advertise a unique service name and additional metadata (e.g., username, capabilities) via TXT records. I updated the peer management logic to:

- Parse and store service names and metadata.
- Ensure uniqueness (e.g., by including a random suffix or hash in the service name).

### 5. Network Debugging

mDNS relies on multicast, which can be blocked or filtered by routers/firewalls. Debugging required:

- Using `tcpdump`/`wireshark` to inspect multicast traffic. For example:
  - `sudo tcpdump -i any port 5353`
- Verifying that the correct multicast group (`224.0.0.251:5353`) was joined.
- Testing on different OSes (Linux, macOS, Windows) to catch platform-specific issues.
- Checking firewall rules and router settings if discovery failed.

### 6. Testing and Edge Cases

- Verified peer discovery and chat messaging on various LAN setups.
- Handled cases where mDNS was unavailable (e.g., fallback to broadcast or manual peer entry).
- Ensured graceful handling of peer disconnects and network partitions.

---

## Example: mDNS Integration (Rust)

```rust
use mdns::{RecordKind, Record};
use std::time::Duration;

const SERVICE_NAME: &str = "_chat._udp.local";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Discover peers advertising the chat service
    let stream = mdns::discover::all(SERVICE_NAME, Duration::from_secs(15)).listen();
    futures_util::pin_mut!(stream);
    while let Some(Ok(response)) = stream.next().await {
        for record in response.records() {
            if let RecordKind::A(addr) = record.kind {
                println!("Discovered peer at {}", addr);
                // Here you could initiate a TCP connection, etc.
            }
        }
    }
    Ok(())
}
```

---

## Real-World Troubleshooting

- **Multicast not working?** Check your router and firewall settings. Some WiFi networks (especially guest networks) block multicast by default.
- **Peers not discovered?** Use `tcpdump` or `wireshark` to verify mDNS packets are being sent and received.
- **Cross-platform quirks:** mDNS behavior can differ between Linux, macOS, and Windows. Test on all platforms you intend to support.

---

## Lessons Learned

- **Abstraction is essential:** Decoupling peer discovery from the rest of the app made it easier to swap implementations.
- **Async Rust is powerful but tricky:** Handling async events and lifetimes required careful design.
- **Networking is full of edge cases:** Real-world networks are unpredictable; robust error handling is a must.
- **The Rust ecosystem is maturing:** Libraries like `libp2p` make building P2P apps much more approachable.

---

## Further Reading & Resources

- [`libmdns` crate](https://crates.io/crates/libmdns)
- [`mdns` crate](https://crates.io/crates/mdns)
- [mDNS RFC 6762](https://datatracker.ietf.org/doc/html/rfc6762)
- [Rust async book](https://rust-lang.github.io/async-book/)
- [Wireshark mDNS analysis](https://wiki.wireshark.org/MDNS)

---

## Conclusion & Next Steps

Migrating from UDP broadcast to mDNS made the chat app more robust, scalable, and user-friendly. The process deepened my understanding of network programming and async Rust. If you’re building a P2P app, consider mDNS for local peer discovery—and be ready for some networking adventures!

If you’ve tried something similar, or have tips or stories to share, I’d love to hear from you! Want to try it yourself or contribute? [Check out the repository](#) or reach out with questions!
