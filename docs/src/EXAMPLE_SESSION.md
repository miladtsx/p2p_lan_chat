# 🎬 Example Session Walkthrough

This walkthrough shows how P2P Chat works with multiple peers.

## Terminal 1 - Alice

```
$ cargo run -- start --port 8080 --name Alice

🎙️  Starting P2P Chat...
👤 Your ID: abc123-def456-ghi789
📡 Your Name: Alice
🔌 Listening on port: 8080
🔗 TCP listener started on port 8080

💬
```

## Terminal 2 - Bob (started 5 seconds later)

```
$ cargo run -- start --port 8081 --name Bob

🎙️  Starting P2P Chat...
👤 Your ID: xyz789-uvw456-rst123
📡 Your Name: Bob
🔌 Listening on port: 8081
🔗 TCP listener started on port 8081

💬
```

## Peer Discovery (Alice's terminal)

```
🔍 Discovered new peer: Bob (192.168.1.101)
💬 /list
👥 Discovered peers:
  - Bob (xyz789-uvw456-rst123) at 192.168.1.101:8081
```

## Messaging from Alice to Bob
```
💬 Hello Bob!
📤 Message sent to 1 peer(s)
```

## Bob Receives the Message

```
📨 Alice says: Hello Bob!
💬 Hey Alice! Nice to meet you!
📤 Message sent to 1 peer(s)
```

## Alice Receives Bob's Reply

```
📨 Bob says: Hey Alice! Nice to meet you!
💬
```

## Adding Charlie (Terminal 3)

```
$ cargo run -- start --port 8082 --name Charlie

🎙️  Starting P2P Chat...
👤 Your ID: pqr456-stu789-vwx123
📡 Your Name: Charlie

# After discovery...
💬 /list
👥 Discovered peers:
  - Alice (abc123-def456-ghi789) at 192.168.1.100:8080
  - Bob (xyz789-uvw456-rst123) at 192.168.1.101:8081

💬 Hello everyone!
📤 Message sent to 2 peer(s)
```

## All Terminals Receive Charlie's Message

Alice's terminal:

```
🔍 Discovered new peer: Charlie (192.168.1.102)
📨 Charlie says: Hello everyone!
```

## Network Diagram

```
    Alice (8080)
        |  \\
        |   \\
        |    \\
    Bob (8081)---Charlie (8082)

UDP Discovery: All peers broadcast on port 9999
TCP Messages: Direct peer-to-peer connections
```

## Key Observations

- Peers auto-discover via UDP
- Messages are sent directly over TCP
- No central server
- Secure messaging and threshold voting supported
- CLI commands for messaging, proposals, voting, and status
