#!/bin/bash

echo "🧪 Basic UDP Test for DJ-4LED"
echo "============================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    pkill -f "cargo run" 2>/dev/null
    pkill -f "led-visualizer" 2>/dev/null
    pkill -f "udp_client" 2>/dev/null
    exit
}

trap cleanup EXIT INT TERM

# Check directory
if [ ! -f "apps/backend/Cargo.toml" ]; then
    echo -e "${RED}Error: Must run from dj-4led root directory${NC}"
    exit 1
fi

cd apps/backend

# Build first
echo -e "${YELLOW}Building backend...${NC}"
cargo build --release --bin led-visualizer --example udp_client >/dev/null 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

# Start server in background
echo -e "\n${YELLOW}Starting UDP server...${NC}"
cargo run --release 2>&1 | grep -E "(UDP|8081|📨|📦|client|packet)" &
SERVER_PID=$!

# Wait for server
echo -e "${YELLOW}Waiting for server to start...${NC}"
sleep 3

# Check if port is open
if lsof -i :8081 >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Server is listening on port 8081${NC}"
else
    echo -e "${RED}❌ Server failed to start${NC}"
    exit 1
fi

# Run client for 15 seconds
echo -e "\n${YELLOW}Starting UDP client for 15 seconds...${NC}"
echo -e "${YELLOW}Watch for connection messages above ↑${NC}\n"

cargo run --release --example udp_client 2>&1 | grep -E "(Bound|Sent|Received|Frame|Spectrum|command)" &
CLIENT_PID=$!

# Let it run for 15 seconds
sleep 15

# Kill the client
kill $CLIENT_PID 2>/dev/null

echo -e "\n${GREEN}Test completed!${NC}"
echo -e "${YELLOW}If you saw '📨 Received' messages above, UDP is working!${NC}"
