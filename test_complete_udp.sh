#!/bin/bash

echo "🎯 Complete UDP Communication Test for DJ-4LED"
echo "============================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to cleanup background processes
cleanup() {
    print_status $YELLOW "\n🛑 Cleaning up..."

    # Kill all related processes
    pkill -f "cargo run --release" 2>/dev/null
    pkill -f "led-visualizer" 2>/dev/null
    pkill -f "udp_client" 2>/dev/null

    # Clean up log files
    rm -f /tmp/udp_server.log /tmp/udp_client.log 2>/dev/null

    print_status $GREEN "✅ Cleanup complete"
    exit
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

# Check if we're in the right directory
if [ ! -f "apps/backend/Cargo.toml" ]; then
    print_status $RED "❌ Error: Must run from dj-4led root directory"
    exit 1
fi

# Build the project
print_status $BLUE "📦 Building backend and UDP client..."
cd apps/backend
cargo build --release --bin led-visualizer --example udp_client

if [ $? -ne 0 ]; then
    print_status $RED "❌ Build failed!"
    exit 1
fi

print_status $GREEN "✅ Build successful"

# Start the server
print_status $BLUE "\n🚀 Starting UDP server on port 8081..."
cargo run --release 2>&1 | while IFS= read -r line; do
    echo "[SERVER] $line"
    # Save important lines to log
    echo "$line" | grep -E "(UDP|8081|📨|📦|Connect|client)" >> /tmp/udp_server.log
done &
SERVER_PID=$!

# Wait for server to start
print_status $YELLOW "⏳ Waiting for server to initialize..."
sleep 5

# Check if server started successfully
if ! lsof -i :8081 > /dev/null 2>&1; then
    print_status $RED "❌ Server failed to bind to port 8081!"
    print_status $YELLOW "📋 Server output:"
    tail -20 /tmp/udp_server.log
    exit 1
fi

print_status $GREEN "✅ Server is listening on UDP port 8081"

# Test basic UDP connectivity
print_status $BLUE "\n🔌 Testing basic UDP connectivity..."
echo -n -e "\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00" | nc -u -w1 localhost 8081
sleep 1

# Start the client
print_status $BLUE "\n🎮 Starting UDP client..."
timeout 30 cargo run --release --example udp_client 2>&1 | while IFS= read -r line; do
    echo "[CLIENT] $line"
    echo "$line" >> /tmp/udp_client.log
done &
CLIENT_PID=$!

# Monitor for 30 seconds
print_status $YELLOW "\n📊 Monitoring communication for 30 seconds..."
SECONDS=0
while [ $SECONDS -lt 30 ]; do
    # Check server logs for received packets
    SERVER_PACKETS=$(grep -c "📨 Received" /tmp/udp_server.log 2>/dev/null || echo "0")
    CLIENT_FRAMES=$(grep -c "Frame:" /tmp/udp_client.log 2>/dev/null || echo "0")

    printf "\r⏱️  Time: %02d/30s | Server packets: %d | Client frames: %d" $SECONDS $SERVER_PACKETS $CLIENT_FRAMES

    sleep 1
done

echo "" # New line after progress

# Wait for client to finish
wait $CLIENT_PID

# Analysis
print_status $BLUE "\n📈 Analyzing results..."

# Check server received packets
SERVER_PACKETS=$(grep -c "📨 Received" /tmp/udp_server.log 2>/dev/null || echo "0")
SERVER_CLIENTS=$(grep -c "New UDP client connected" /tmp/udp_server.log 2>/dev/null || echo "0")
SERVER_COMMANDS=$(grep -c "📦 Packet type: Command" /tmp/udp_server.log 2>/dev/null || echo "0")

# Check client statistics
CLIENT_FRAMES=$(grep -c "Frame:" /tmp/udp_client.log 2>/dev/null || echo "0")
CLIENT_SPECTRUMS=$(grep -c "Spectrum:" /tmp/udp_client.log 2>/dev/null || echo "0")
CLIENT_ACKS=$(grep -c "Received ACK" /tmp/udp_client.log 2>/dev/null || echo "0")

print_status $YELLOW "\n📊 Test Results:"
echo "================"
echo "Server Statistics:"
echo "  - Total packets received: $SERVER_PACKETS"
echo "  - Clients connected: $SERVER_CLIENTS"
echo "  - Commands processed: $SERVER_COMMANDS"
echo ""
echo "Client Statistics:"
echo "  - Frames received: $CLIENT_FRAMES"
echo "  - Spectrum data received: $CLIENT_SPECTRUMS"
echo "  - ACKs received: $CLIENT_ACKS"

# Determine test result
if [ $SERVER_PACKETS -gt 0 ] && [ $CLIENT_FRAMES -gt 0 ]; then
    print_status $GREEN "\n✅ UDP communication test PASSED!"
    print_status $GREEN "The server and client are communicating successfully."
else
    print_status $RED "\n❌ UDP communication test FAILED!"

    if [ $SERVER_PACKETS -eq 0 ]; then
        print_status $RED "Server did not receive any packets from client."
    fi

    if [ $CLIENT_FRAMES -eq 0 ]; then
        print_status $RED "Client did not receive any frames from server."
    fi

    print_status $YELLOW "\n📋 Last server logs:"
    tail -10 /tmp/udp_server.log

    print_status $YELLOW "\n📋 Last client logs:"
    tail -10 /tmp/udp_client.log
fi

print_status $BLUE "\n💡 Tips:"
echo "  - To run server only: cd apps/backend && cargo run --release"
echo "  - To run client only: cd apps/backend && cargo run --release --example udp_client"
echo "  - Server listens on UDP port 8081"
echo "  - Check UDP_PROTOCOL.md for protocol documentation"
