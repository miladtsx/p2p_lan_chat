#!/bin/bash

# P2P Chat Demo Script
# This script demonstrates the P2P functionality by running multiple instances

echo "🎙️ P2P Chat Demo"
echo "=========================="
echo ""

# Build the application first
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
    pkill -f "p2p_Chat"
    exit 0
}

# Set trap to cleanup on Ctrl+C
trap cleanup SIGINT SIGTERM

echo "🚀 Starting demo instances..."
echo ""
echo "Instructions:"
echo "1. Wait for peers to discover each other (about 5 seconds)"
echo "2. Type messages in any terminal window"
echo "3. Use /list to see discovered peers"
echo "4. Use /quit to exit an instance"
echo "5. Press Ctrl+C here to stop all instances"
echo ""

# Start multiple instances in background with different ports
echo "Starting Alice on port 8080..."
gnome-terminal --title="Alice (Port 8080)" -- bash -c "
    cd $(pwd)
    ./target/release/p2p_chat start --port 8080 --name Alice
    read -p 'Press Enter to close...'
" 2>/dev/null &

sleep 1

echo "Starting Bob on port 8081..."
gnome-terminal --title="Bob (Port 8081)" -- bash -c "
    cd $(pwd)
    ./target/release/p2p_chat start --port 8081 --name Bob
    read -p 'Press Enter to close...'
" 2>/dev/null &

sleep 1

echo "Starting Charlie on port 8082..."
gnome-terminal --title="Charlie (Port 8082)" -- bash -c "
    cd $(pwd)
    ./target/release/p2p_chat start --port 8082 --name Charlie
    read -p 'Press Enter to close...'
" 2>/dev/null &

echo ""
echo "✅ All instances started!"
echo "📱 Check the terminal windows that opened"
echo "🔄 Peers should discover each other within 5 seconds"
echo ""
echo "💡 Try these commands in any terminal:"
echo "   - Type 'Hello everyone!' to broadcast"
echo "   - Type '/list' to see peers"
echo "   - Type '/quit' to exit that instance"
echo ""
echo "Press Ctrl+C to stop all instances..."

# Wait for user to stop
while true; do
    sleep 1
done
