#!/bin/bash

echo "🧪 Testing UDP Communication for DJ-4LED"
echo "========================================"

# Function to cleanup background processes
cleanup() {
    echo -e "\n🛑 Cleaning up..."
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null
    fi
    if [ ! -z "$CLIENT_PID" ]; then
        kill $CLIENT_PID 2>/dev/null
    fi
    exit
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

# Change to backend directory
cd apps/backend

# Build the project
echo "📦 Building the backend..."
cargo build --release --example udp_client

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

# Start the server
echo -e "\n🚀 Starting UDP server..."
cargo run --release 2>&1 | while read line; do
    echo "[SERVER] $line"
done &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to initialize..."
sleep 5

# Check if server is running
if ! ps -p $SERVER_PID > /dev/null; then
    echo "❌ Server failed to start!"
    exit 1
fi

echo "✅ Server is running (PID: $SERVER_PID)"

# Start the client
echo -e "\n🔌 Starting UDP client..."
timeout 30 cargo run --release --example udp_client 2>&1 | while read line; do
    echo "[CLIENT] $line"
done &
CLIENT_PID=$!

# Wait for client to finish or timeout
wait $CLIENT_PID
CLIENT_EXIT=$?

if [ $CLIENT_EXIT -eq 124 ]; then
    echo -e "\n⏰ Client timeout after 30 seconds"
else
    echo -e "\n✅ Client completed with exit code: $CLIENT_EXIT"
fi

# Give some time to see final messages
sleep 2

echo -e "\n📊 Test Summary:"
echo "================"
echo "Server PID: $SERVER_PID"
echo "Client ran for 30 seconds"
echo ""
echo "To run longer tests:"
echo "  Terminal 1: cd apps/backend && cargo run --release"
echo "  Terminal 2: cd apps/backend && cargo run --release --example udp_client"
