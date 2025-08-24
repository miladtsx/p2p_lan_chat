# P2P Chat Documentation

A concise guide to the P2P Chat project in Rust.

## Features
- Automatic peer discovery (UDP)
- Real-time messaging (TCP)
- Decentralized, no server
- CLI interface
- Heartbeat system
- Threshold voting for secure-only messaging
- Ed25519 cryptographic security

## How It Works
- Peers broadcast presence via UDP
- Direct TCP connections for messages
- CLI commands for messaging, proposals, voting, and status
- Secure-only mode: unsigned messages rejected after threshold approval

## Design Pattern

Message handling uses the Command Pattern. Each message type implements a trait for polymorphic dispatch, making it easy to add new types.

### Adding a New Message Type

- Add your variant to `NetworkMessage`.
- Implement the `NetworkCommand` trait for your type.
- Update the command factory to support your new type.