#!/bin/bash

# Script to test the UDP client example

echo "🚀 Testing UDP Client for DJ-4LED"
echo "================================"

# Change to backend directory
cd apps/backend

# Build the backend with UDP server
echo "📦 Building backend with UDP server..."
cargo build --release

# Check if build was successful
if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

# Start the backend in the background
echo "🎵 Starting backend server..."
cargo run --release &
BACKEND_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 3

# Run the UDP client example
echo "🔌 Starting UDP client example..."
cargo run --example udp_client

# When client exits, kill the backend
echo "🛑 Stopping backend server..."
kill $BACKEND_PID 2>/dev/null

echo "✅ Test complete!"
