# 🎙️ P2P Chat

A text-based peer-to-peer Chat room built in Rust demonstrating P2P networking concepts including **peer discovery** and **message broadcasting**.

## 🚀 Features

- **Automatic Peer Discovery**: Discovers other Chat instances on the local network
- **Real-time Messaging**: Send and receive messages instantly
- **Decentralized**: No central server required - peers communicate directly in LAN
- **Simple CLI**: Easy-to-use command line interface
- **Heartbeat System**: Keeps track of active peers

## 🛠️ Installation

Make sure you have Rust installed, then clone and build:

```bash
git clone <your-repo>
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

- **Send a message**: Just type your message and press Enter
- **`/msg <message>`**: Alternative way to send a message
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

## 🐛 Troubleshooting

### Peers Not Discovering Each Other

- Make sure all instances are on the same network
- Check firewall settings (UDP port 9999, TCP ports)
- Verify different ports are used for each instance

### Messages Not Sending

- Ensure peers have been discovered first (use `/list`)
- Check that TCP ports are not blocked by firewall
- Try restarting instances if connections seem stuck

### Permission Issues

On some systems you might need to run with elevated privileges for UDP broadcast
