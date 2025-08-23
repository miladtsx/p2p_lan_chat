# Ed25519 Message Signing and Verification in P2P Chat

This document explains how to use the cryptographic message signing and verification features implemented in the P2P chat application using Ed25519 digital signatures.

## Overview

The P2P chat application now includes cryptographic message signing and verification to ensure:
- **Message Authenticity**: Messages are guaranteed to come from the claimed sender
- **Message Integrity**: Messages cannot be tampered with in transit
- **Replay Attack Prevention**: Messages include timestamps to prevent replay attacks
- **Identity Verification**: Each peer has a unique cryptographic identity

## How It Works

### 1. Cryptographic Identity Generation

When a peer starts up, it automatically generates a fresh Ed25519 keypair:
- **Private Key**: Used to sign outgoing messages (never shared)
- **Public Key**: Used to verify incoming messages (shared with other peers)

```rust
// Each peer gets a unique cryptographic identity
let crypto_manager = CryptoManager::new(peer_id, name);
```

### 2. Message Signing

When broadcasting a message, the peer automatically signs it with their private key:

```rust
// Message is automatically signed before broadcasting
peer.broadcast_message("Hello, world!").await?;
```

The signing process:
1. Creates a message hash that includes content + timestamp
2. Signs the hash with the peer's Ed25519 private key
3. Attaches the signature and public key to the message
4. Sends both signed and regular message formats for compatibility

### 3. Message Verification

When receiving a message, peers automatically verify the signature:

```rust
// Messages are automatically verified upon receipt
// Verified messages show: 🔐 Alice says (verified): Hello, world!
// Invalid signatures show: ⚠️  Alice says (INVALID SIGNATURE): Hello, world!
```

The verification process:
1. Extracts the signature and public key from the message
2. Reconstructs the original message hash (content + timestamp)
3. Verifies the signature using the sender's public key
4. Caches the public key for future verifications

## Usage Examples

### Starting a Peer with Cryptographic Identity

```bash
# Start a peer - cryptographic identity is automatically generated
cargo run -- --name Alice --port 8080
```

You'll see output like:
```
🎙️  Starting P2P Chat...
👤 Your ID: 123e4567-e89b-12d3-a456-426614174000
📡 Your Name: Alice
🔌 Listening on port: 8080
🔐 Your Public Key: a1b2c3d4e5f6...
🔐 Full Key: a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6
```

### Sending Messages

#### Signed Messages (Default)
All regular messages are automatically signed:

```
💬 Hello, this message will be cryptographically signed!
📤 Signed message sent to 2 peer(s)
🔐 Message signed with Ed25519 for authenticity
```

#### Unsigned Messages
You can also send messages without cryptographic signing:

```
💬 /unsigned This message will not be signed
📤 Unsigned message sent to 2 peer(s)
⚠️  Message sent without cryptographic signature
```

**Note**: Use unsigned messages when you want to:
- Send messages quickly without cryptographic overhead
- Test network connectivity
- Send messages that don't require authenticity verification
- Maintain backward compatibility with peers that don't support signatures

### Viewing Cryptographic Information

Use the `/crypto` command to see your cryptographic identity:

```
💬 /crypto
🔐 Cryptographic Identity:
  Peer ID: 123e4567-e89b-12d3-a456-426614174000
  Name: Alice
  Public Key: a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6
  Known Peer Keys: 3
```

### Receiving and Verifying Messages

When you receive messages from other peers, they're automatically verified:

```
🔐 Bob says (verified): Hello Alice!
⚠️  Charlie says (INVALID SIGNATURE): This message was tampered with
❓ Dave says (verification failed: Invalid signature): Hello
📝 Eve says (unsigned): This message has no signature
```

The verification process:
1. Extracts the signature and public key from the message
2. Reconstructs the original message hash (content + timestamp)
3. Verifies the signature using the sender's public key
4. Caches the public key for future verifications

### Message Display Indicators

Different message types are displayed with distinct indicators:

- **🔐 Verified Messages**: Cryptographically signed and verified
- **⚠️ Invalid Signature**: Signed but verification failed (possible tampering)
- **❓ Verification Failed**: Signed but verification process failed
- **📝 Unsigned Messages**: No cryptographic signature attached

## Security Features

### 1. Replay Attack Prevention

Messages could include timestamps and are rejected if too old:

```rust
// Messages older than 1 hour are automatically rejected
if !crypto_manager.is_message_recent(timestamp, 3600) {
    return Err(CryptoError::MessageTooOld);
}
```

### 2. Public Key Caching

Peer public keys are automatically cached after first verification:

```rust
// Public keys are cached for efficient future verifications
crypto_manager.add_known_peer(peer_id, public_key).await?;
```

### 3. Automatic Identity Exchange

When peers discover each other, they automatically exchange public keys:

```
🔐 Added public key for peer Bob: a1b2c3d4
🔐 Identity announced to 1 peer(s)
```

## Message Formats

### Regular Message (with optional signature)
```json
{
  "from_id": "peer-uuid",
  "from_name": "Alice",
  "content": "Hello, world!",
  "timestamp": 1234567890,
  "signature": "base64-signature",
  "public_key": "base64-public-key"
}
```

### Signed Message (cryptographically verified)
```json
{
  "signer_id": "peer-uuid",
  "signer_name": "Alice",
  "message": "Hello, world!",
  "timestamp": 1234567890,
  "signature": "base64-signature",
  "public_key": "base64-public-key",
}
```

## Error Handling

The system gracefully handles various cryptographic scenarios:

- **Valid Signature**: Message displayed with 🔐 verification indicator
- **Invalid Signature**: Message displayed with ⚠️ warning indicator
- **Verification Failed**: Message displayed with ❓ error indicator
- **No Signature**: Message displayed normally (backward compatibility)

## Backward Compatibility

The system maintains backward compatibility:
- Peers without cryptographic capabilities can still send/receive messages
- Signed messages are sent alongside regular messages
- The system automatically falls back to regular message format if needed

## Performance Considerations

- **Key Generation**: One-time cost when peer starts
- **Message Signing**: Minimal overhead for outgoing messages
- **Message Verification**: Fast Ed25519 verification for incoming messages
- **Key Caching**: Public keys are cached to avoid repeated parsing

## Best Practices

1. **Keep Private Keys Secure**: Never share or expose your private key
2. **Verify Message Sources**: Always check the verification status of received messages
3. **Monitor for Tampering**: Pay attention to invalid signature warnings
4. **Regular Key Rotation**: Consider regenerating keys periodically for enhanced security
5. **Network Security**: Use the cryptographic features in addition to network-level security

## Troubleshooting

### Common Issues

1. **"verification failed" errors**: Usually indicate corrupted messages or network issues
2. **"INVALID SIGNATURE" warnings**: Indicate message tampering or key mismatch
3. **Missing public keys**: Peers will automatically exchange keys upon discovery

### Debugging

Enable debug logging to see detailed cryptographic operations:

```bash
RUST_LOG=debug cargo run -- --name Alice --port 8080
```

## Technical Details

### Ed25519 Implementation

- Uses the `ed25519-dalek` crate for cryptographic operations
- 256-bit private keys, 256-bit public keys
- Deterministic signatures for consistent verification
- Fast verification suitable for real-time messaging

### Key Management

- Keys are generated using cryptographically secure random number generation
- Public keys are shared via mDNS discovery and TCP connections
- Private keys are never transmitted over the network
- Key rotation can be implemented by restarting the peer

### Message Signing Process

1. Create message hash: `hash(content + ":" + timestamp)`
2. Sign hash with Ed25519 private key
3. Attach signature and public key to message
4. Broadcast both signed and regular formats

### Message Verification Process

1. Extract signature and public key from message
2. Reconstruct original message hash
3. Verify signature using Ed25519 public key
4. Cache public key for future use
5. Display verification status to user
