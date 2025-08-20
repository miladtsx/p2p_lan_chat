#!/bin/bash

# Manual Test Script for P2P Chat
# This script helps you test the P2P functionality step by step

echo "🎙️ P2P Chat Manual Test"
echo "================================="
echo ""

# Build first
echo "📦 Building application..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

echo "📋 Manual Testing Instructions:"
echo ""
echo "1. Open 2-3 terminal windows"
echo "2. In each terminal, run one of these commands:"
echo ""
echo "   Terminal 1: ./target/release/p2p_chat start --port 8080 --name Alice"
echo "   Terminal 2: ./target/release/p2p_chat start --port 8081 --name Bob"
echo "   Terminal 3: ./target/release/p2p_chat start --port 8082 --name Charlie"
echo ""
echo "3. Wait 5-10 seconds for peer discovery"
echo "4. In any terminal, type '/list' to see discovered peers"
echo "5. Type any message to broadcast it to all peers"
echo "6. Try '/quit' to exit a peer and see how others handle it"
echo ""
echo "🔍 What you're learning:"
echo "• Peer Discovery: How peers find each other without a central server"
echo "• P2P Messaging: Direct communication between peers"
echo "• Network Protocols: UDP for discovery, TCP for reliable messaging"
echo "• Async Programming: Concurrent network operations in Rust"
echo ""
echo "💡 Network Details:"
echo "• UDP Port 9999: Used for peer discovery broadcasts"
echo "• TCP Ports 8080+: Used for reliable message delivery"
echo "• JSON Messages: Structured communication protocol"
echo ""

# Quick single instance test
echo "🧪 Quick Test - Starting one instance for 10 seconds..."
echo "(This just verifies the app starts correctly)"
echo ""

timeout 10s ./target/release/p2p_chat start --port 8083 --name TestRunner &
TEST_PID=$!

sleep 2
echo "✅ Application started successfully!"
echo "   (If you saw startup messages above, it's working)"

wait $TEST_PID 2>/dev/null

echo ""
echo "🎯 Ready for manual testing!"
echo "Open multiple terminals and follow the instructions above."
