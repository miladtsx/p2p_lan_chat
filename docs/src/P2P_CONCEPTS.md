# 🌐 Peer-to-Peer Networking Concepts

## What is P2P?
Distributed architecture: peers act as both clients and servers. No central authority.

## Key Concepts
- Peer Discovery: UDP broadcast, decentralized
- Direct Communication: TCP connections
- Broadcast Messaging: Send to all peers
- JSON Protocol: Structured messages
- Network Resilience: Heartbeat system

## P2P vs Client-Server
| Aspect           | P2P Chat         | Client-Server         |
|------------------|------------------|----------------------|
| Architecture     | Decentralized    | Centralized          |
| Failure Points   | No single point  | Server failure breaks|
| Scalability      | Scales with peers| Limited by server    |
| Discovery        | Peer-to-peer     | Central directory    |
| Communication    | Direct           | Through server       |

## Technical Details
- UDP for discovery
- TCP for messaging
- Async Rust (tokio)
- Ed25519 signatures

## Real-World Examples
- BitTorrent, Skype, Bitcoin, IPFS

## Next Steps
- Encryption, file sharing, private messaging, mobile support