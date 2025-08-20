#!/bin/bash

# Quick Demo Script for P2P Chat
# This script starts two instances to demonstrate P2P functionality

echo "🎙️ P2P Chat Quick Demo"
echo "==============================="
echo ""

# Build the application
echo "📦 Building..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build complete!"
echo ""

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "🧹 Stopping demo..."
    kill $ALICE_PID $BOB_PID 2>/dev/null
    wait 2>/dev/null
    echo "👋 Demo stopped!"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Start Alice in background
echo "🚀 Starting Alice on port 8080..."
./target/release/p2p_chat start --port 8080 --name Alice > alice.log 2>&1 &
ALICE_PID=$!

# Wait a moment
sleep 2

# Start Bob in background
echo "🚀 Starting Bob on port 8081..."
./target/release/p2p_chat start --port 8081 --name Bob > bob.log 2>&1 &
BOB_PID=$!

echo ""
echo "✅ Both peers started!"
echo ""
echo "📋 Demo Instructions:"
echo "1. Wait 5-10 seconds for peer discovery"
echo "2. Check the log files to see what's happening:"
echo "   - Alice's log: tail -f alice.log"
echo "   - Bob's log: tail -f bob.log"
echo ""
echo "3. To send a test message, run:"
echo "   echo 'Hello from script!' | nc localhost 8080"
echo ""
echo "💡 You can also open terminals and run manually:"
echo "   ./target/release/p2p_chat start --port 8082 --name Charlie"
echo ""
echo "🛑 Press Ctrl+C to stop the demo"
echo ""

# Keep the demo running and show status
while true; do
    if ! kill -0 $ALICE_PID 2>/dev/null; then
        echo "⚠️  Alice stopped unexpectedly"
        break
    fi
    if ! kill -0 $BOB_PID 2>/dev/null; then
        echo "⚠️  Bob stopped unexpectedly"
        break
    fi

    echo -n "🔄 Demo running... (Alice: PID $ALICE_PID, Bob: PID $BOB_PID)"
    echo -e "\r"
    sleep 5
done

cleanup
