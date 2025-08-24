# 🎙️ P2P Chat

A text-based peer-to-peer chat built in Rust, showcasing **P2P networking**, **cryptographic messaging**, and **threshold security**.

## 🚀 Features

- **Automatic Peer Discovery**: LAN peers detected automatically via UDP broadcast
- **Real-time Messaging**: Instant text delivery over TCP
- **Decentralized**: No central server required
- **Simple CLI**: Easy-to-use command line interface
- **Heartbeat System**: Tracks active peers
- **Threshold Signatures**: M-of-N voting for enabling secure-only messaging
- **Cryptographic Security**: Ed25519 message signing & verification
- **Secure-Only Mode**: Reject unsigned messages once enabled

## 🛠️ Installation

Make sure you have Rust installed, then clone and build:

```bash
git clone https://github.com/miladtsx/p2p_lan_chat
cd p2p
cargo build
```

## 📡 Usage

### Start a Peer

```bash
cargo run -- start --name "Alice" --port 8080
```

Default: `name="Anonymous"`, `port=8080`.

### CLI Commands

| Command                | Description                      |                    |
| ---------------------- | -------------------------------- | ------------------ |
| Type message           | Broadcast a signed message       |                    |
| `/msg <msg>`           | Send a signed message            |                    |
| `/unsigned <msg>`      | Send unsigned message            |                    |
| `/propose <desc>`      | Propose secure-only messaging    |                    |
| `/vote <id> <vote>`    | <approve or reject>              | Vote on a proposal |
| `/proposals`           | List proposals                   |                    |
| `/status`              | Show security & proposal status  |                    |
| `/list`                | List discovered peers            |                    |
| `/crypto`              | Show your cryptographic identity |                    |
| `/quit`                | Exit                             |                    |


### Multi-Peer Testing

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

## 🔧 Architecture

- **Discovery**: UDP `255.255.255.255:9999` broadcast
- **Messaging**: TCP `8080+`, JSON `NetworkMessage`
- **Async Rust**: Concurrent networking with `tokio`
- **Cryptography**: Ed25519 signatures for authenticity & integrity
- **Threshold Voting**: M-of-N approval for secure mode
- **Command Pattern**: Message handling uses a trait-based command dispatch for extensibility and clean code.

### 🧩 Extending Message Types

To add a new message type:
1. Define your variant in `NetworkMessage`.
2. Implement the `NetworkCommand` trait for your type in `src/network/command.rs`.
3. Update the factory function to return your new command.
This design makes it easy to add new features and keep message handling modular.

### Core Components

1. **Peer Discovery**: Broadcasts info every 5s, updates peer list with heartbeats
2. **Message Broadcasting**: TCP delivery, JSON format
3. **Threshold Signature System**: Peer proposals → votes → automatic enforcement of secure-only messaging
4. **Cryptographic Identity**: Auto-generated Ed25519 keypair per peer

### Message Signing & Verification
- Messages signed automatically with private key
- Public key attached for verification
- Unsigned messages allowed for testing/backward compatibility
- Replay prevention using timestamps
- Public keys cached for efficiency

### Indicators in CLI:
| Symbol | Meaning             |
| ------ | ------------------- |
| 🔐     | Verified signature  |
| ⚠️     | Invalid signature   |
| ❓     | Verification failed |
| 📝     | Unsigned            |


```sh
# Send signed message
Hello world!

# Propose secure-only mode
/propose Enable secure-only messaging

# Vote
/vote <proposal_id> approve|reject
```

## 🐛 Troubleshooting

### Peers Not Discovering Each Other

- **Peers not discovering**: Check LAN, firewall, ports
- **Messages not sending**: Ensure peers discovered, TCP open
- **Threshold issues**: Verify proposal ID, check /status

### ⚡ Technical Notes

- Rust crates: `tokio`, `serde`, `serde_json`, `clap`, `uuid`, `local-ip-address`, `ed25519-dalek`
- **Message Format** (JSON):

```json
{
  "from_id": "uuid",
  "from_name": "Alice",
  "content": "Hello",
  "timestamp": 1234567890,
  "signature": "base64",
  "public_key": "base64"
}
```
- Private keys never transmitted; public keys exchanged automatically
- Minimal cryptographic overhead for real-time messaging