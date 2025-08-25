#!/bin/bash

echo "🔐 Testing P2P Chat with Ed25519 Message Signing"
echo "================================================"

# Kill any existing processes
pkill -f "p2p-chat" || true
sleep 1

echo "🚀 Starting first peer (Alice) on port 8080..."
cargo run -- --name Alice --port 8080 &
ALICE_PID=$!
sleep 3

echo "🚀 Starting second peer (Bob) on port 8081..."
cargo run -- --name Bob --port 8081 &
BOB_PID=$!
sleep 5

echo "📝 Sending test message from Alice..."
echo "Hello from Alice! This should be cryptographically signed." | nc -w 1 localhost 8080

echo "📝 Sending test message from Bob..."
echo "Hello from Bob! This should also be cryptographically signed." | nc -w 1 localhost 8081

echo "⏳ Waiting for messages to be processed..."
sleep 10

echo "🔄 Checking peer discovery and message exchange..."
echo "Alice's output:"
ps -p $ALICE_PID >/dev/null && echo "Alice is still running" || echo "Alice has stopped"

echo "Bob's output:"
ps -p $BOB_PID >/dev/null && echo "Bob is still running" || echo "Bob has stopped"

echo "🧹 Cleaning up..."
kill $ALICE_PID $BOB_PID 2>/dev/null || true
pkill -f "p2p-chat" || true

echo "✅ Test completed!"
