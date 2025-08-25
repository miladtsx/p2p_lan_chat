#!/bin/bash

# Test script for unsigned messages feature
echo "🧪 Testing Unsigned Messages Feature"
echo "===================================="
echo ""

# Build the application
echo "📦 Building the application..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Function to cleanup processes on exit
cleanup() {
    echo ""
    echo "🧹 Cleaning up..."
    pkill -f "p2p_chat"
    exit 0
}

# Set trap to cleanup on Ctrl+C
trap cleanup SIGINT SIGTERM

echo "🚀 Starting test instances..."
echo ""
echo "Instructions:"
echo "1. Wait for peers to discover each other (about 5 seconds)"
echo "2. In Alice's terminal, try:"
echo "   - Type 'Hello everyone!' (signed message)"
echo "   - Type '/unsigned This is unsigned!' (unsigned message)"
echo "3. In Bob's terminal, observe the difference:"
echo "   - Signed messages show: 🔐 Alice says (verified): ..."
echo "   - Unsigned messages show: 📝 Alice says (unsigned): ..."
echo "4. Use /quit to exit instances"
echo "5. Press Ctrl+C here to stop all instances"
echo ""

# Start Alice (will send both signed and unsigned messages)
echo "Starting Alice on port 8080..."
gnome-terminal --title="Alice (Port 8080)" -- bash -c "
    cd $(pwd)
    ./target/release/p2p_chat start --port 8080 --name Alice
    read -p 'Press Enter to close...'
" 2>/dev/null &

sleep 2

# Start Bob (will receive and display messages)
echo "Starting Bob on port 8081..."
gnome-terminal --title="Bob (Port 8081)" -- bash -c "
    cd $(pwd)
    ./target/release/p2p_chat start --port 8081 --name Bob
    read -p 'Press Enter to close...'
" 2>/dev/null &

echo ""
echo "✅ Test instances started!"
echo "📱 Check the terminal windows that opened"
echo "🔄 Peers should discover each other within 5 seconds"
echo ""
echo "💡 Test the unsigned message feature:"
echo "   - In Alice's terminal, type '/unsigned Test unsigned message'"
echo "   - In Bob's terminal, you should see: 📝 Alice says (unsigned): Test unsigned message"
echo "   - Compare with regular signed messages"
echo ""
echo "Press Ctrl+C to stop all instances..."

# Wait for user to stop
wait
