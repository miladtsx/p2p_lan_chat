# 🎙️ P2P Chat

A text-based peer-to-peer Chat room built in Rust demonstrating P2P networking concepts including **peer discovery** and **message broadcasting**.

## 🚀 Features

- **Automatic Peer Discovery**: Discovers other Chat instances on the local network
- **Real-time Messaging**: Send and receive messages instantly
- **Decentralized**: No central server required - peers communicate directly in LAN
- **Simple CLI**: Easy-to-use command line interface
- **Heartbeat System**: Keeps track of active peers
- **🔐 Threshold Signatures**: M-of-N approval for secure-only messaging upgrades
- **🔒 Cryptographic Security**: Ed25519 message signing and verification
- **🛡️ Secure-Only Mode**: Optional enforcement of signed messages only

## 🛠️ Installation

Make sure you have Rust installed, then clone and build:

```bash
git clone https://github.com/miladtsx/p2p_lan_chat
cd p2p
cargo build --release
```

## 📡 How to Use

### Starting the Chat

```bash

# Build the application
cargo build --release

# Start with default settings (port 8080, name "Anonymous")
cargo run -- start

# Or specify your name and port
cargo run -- start --name "Alice" --port 8080
```

### Commands

Once running, you can use these commands:

- **Send a message**: Just type your message and press Enter (signed by default)
- **`/msg <message>`**: Alternative way to send a signed message
- **`/unsigned <message>`**: Send a message without cryptographic signing
- **`/propose <description>`**: Propose secure-only messaging upgrade
- **`/vote <proposal_id> <approve|reject>`**: Vote on upgrade proposal
- **`/proposals`**: List active upgrade proposals
- **`/status`**: Show security status and proposals
- **`/list`**: Show all discovered peers
- **`/quit`**: Exit the application

### Testing with Multiple Peers

To test the P2P functionality, open multiple terminals and start different instances:

```bash
# Terminal 1
cargo run -- start --name "Alice" --port 8080

# Terminal 2
cargo run -- start --name "Bob" --port 8081

# Terminal 3
cargo run -- start --name "Charlie" --port 8082
```

Wait for peer discovery, then type messages to broadcast!
Each instance will automatically discover the others and you can send messages between them!

## Commands

- Type any message to broadcast it
- `/list` - Show discovered peers
- `/quit` - Exit application

## 🏗️ Architecture

- **UDP port 9999**: Peer discovery broadcasts
- **TCP port 8080+**: Reliable message delivery
- **JSON serialization**: All network messages
- **Async Rust**: Concurrent network operations
- **🔐 Ed25519 cryptography**: Message signing and verification
- **🗳️ Threshold voting**: M-of-N approval for security upgrades

### Core Components

1. **Peer Discovery System**

   - Uses UDP broadcast on port 9999
   - Peers announce themselves every 5 seconds
   - Automatic detection of new peers joining the network

2. **Message Broadcasting**

   - TCP connections for reliable message delivery
   - Messages are sent to all discovered peers
   - JSON serialization for cross-platform compatibility

3. **Heartbeat System**
   - Peers send heartbeats every 10 seconds
   - Helps maintain awareness of active peers

4. **🔐 Threshold Signature System**

   - **Upgrade Proposals**: Any peer can propose enabling secure-only messaging
   - **Voting Mechanism**: Peers vote on proposals with cryptographic signatures
   - **Threshold Enforcement**: M-of-N approvals required (simple majority by default)
   - **Secure-Only Mode**: Once enabled, all messages must be signed and verified
   - **Automatic Enforcement**: System rejects unsigned messages when secure mode is active

### Network Protocols

- **Discovery**: UDP broadcast on `255.255.255.255:9999`
- **Messaging**: TCP connections on specified ports (default 8080)
- **Message Format**: JSON serialized `NetworkMessage` enum

### Data Structures

```rust
struct PeerInfo {
    id: String,        // Unique UUID
    name: String,      // Display name
    ip: IpAddr,        // IP address
    port: u16,         // TCP port for messages
}

struct Message {
    from_id: String,      // Sender's UUID
    from_name: String,    // Sender's display name
    content: String,      // Message content
    timestamp: u64,       // Unix timestamp
}
```

## 🔧 Technical Details

### Dependencies

- **tokio**: Async runtime for handling concurrent network operations
- **serde/serde_json**: Serialization for network messages
- **clap**: Command line argument parsing
- **uuid**: Unique peer identification
- **local-ip-address**: Getting local IP for peer info

### How Peer Discovery Works

1. Each peer broadcasts a `Discovery` message containing their info every 5 seconds
2. All peers listen on UDP port 9999 for discovery messages
3. When a new peer is discovered, it's added to the local peer list
4. Heartbeat messages keep the peer list up to date

### How Messaging Works

1. User types a message in the CLI
2. Message is wrapped in a `NetworkMessage::Chat` variant
3. TCP connections are established to all known peers
4. Message is sent as JSON over each connection
5. Receiving peers display the message in their CLI

## 🔐 Threshold Signature System

The P2P Chat includes a sophisticated threshold signature system that allows the network to democratically upgrade to secure-only messaging. This ensures that no single peer can unilaterally change the security policy.

### How It Works

1. **Proposal Creation**: Any peer can propose enabling secure-only messaging
2. **Voting Round**: All peers vote on the proposal (approve/reject)
3. **Threshold Check**: System calculates if required approvals are met
4. **Automatic Enforcement**: Once approved, all messages must be signed

### Usage Examples

#### Creating an Upgrade Proposal

```bash
# Propose enabling secure-only messaging
/propose Enable secure-only messaging for all future communications

# System will respond with:
# ✅ Upgrade proposal created successfully!
# 📋 Proposal ID: 123e4567-e89b-12d3-a456-426614174000
# 📊 Requires 2/3 approvals to enable
```

#### Voting on Proposals

```bash
# Approve a proposal
/vote 123e4567-e89b-12d3-a456-426614174000 approve

# Reject a proposal
/vote 123e4567-e89b-12d3-a456-426614174000 reject
```

#### Checking Status

```bash
# List active proposals
/proposals

# Show security status
/status
```

### Security Features

- **Cryptographic Signatures**: All votes are signed with Ed25519 keys
- **Duplicate Prevention**: Each peer can only vote once per proposal
- **Threshold Enforcement**: Simple majority rule (configurable)
- **Automatic Mode Switch**: Secure-only mode activates immediately upon approval
- **Message Rejection**: Unsigned messages are rejected when secure mode is active

### Threshold Calculation

The system uses a simple majority rule:
- **Required Approvals**: `(total_peers / 2) + 1`
- **Example**: In a 3-peer network, 2 approvals are required
- **Example**: In a 5-peer network, 3 approvals are required

## 🐛 Troubleshooting

### Peers Not Discovering Each Other

- Make sure all instances are on the same network
- Check firewall settings (UDP port 9999, TCP ports)
- Verify different ports are used for each instance

### Messages Not Sending

- Ensure peers have been discovered first (use `/list`)
- Check that TCP ports are not blocked by firewall
- Try restarting instances if connections seem stuck

### Threshold Signature Issues

- **Proposal not found**: Check the proposal ID with `/proposals`
- **Vote rejected**: Ensure you haven't already voted on this proposal
- **Secure mode not activating**: Verify threshold requirements are met with `/status`

### Permission Issues

On some systems you might need to run with elevated privileges for UDP broadcast
