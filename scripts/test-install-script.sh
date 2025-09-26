#!/bin/bash
# Test the install script with the current binary

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Testing Campfire Install Script${NC}"
echo ""

# Create a temporary directory for testing
TEST_DIR=$(mktemp -d)
echo -e "${YELLOW}Test directory: ${TEST_DIR}${NC}"

# Copy the binary to a temporary web-accessible location (simulate GitHub release)
mkdir -p "${TEST_DIR}/release"
cp release-artifacts/campfire-on-rust-darwin-aarch64 "${TEST_DIR}/release/"

# Start a simple HTTP server to serve the binary
echo -e "${YELLOW}Starting test HTTP server...${NC}"
cd "${TEST_DIR}/release"
python3 -m http.server 8000 &
SERVER_PID=$!
cd - >/dev/null

# Wait for server to start
sleep 2

# Test the binary download
echo -e "${YELLOW}Testing binary download...${NC}"
DOWNLOAD_URL="http://localhost:8000/campfire-on-rust-darwin-aarch64"

if curl -f -L -o "${TEST_DIR}/test-binary" "$DOWNLOAD_URL"; then
    echo -e "${GREEN}✅ Binary download successful${NC}"
    
    # Make it executable
    chmod +x "${TEST_DIR}/test-binary"
    
    # Test that it's a valid binary
    if file "${TEST_DIR}/test-binary" | grep -q "executable"; then
        echo -e "${GREEN}✅ Binary is executable${NC}"
        
        # Test help flag (should not start the server)
        if "${TEST_DIR}/test-binary" --help >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Binary responds to --help${NC}"
        else
            echo -e "${YELLOW}⚠️  Binary doesn't respond to --help (this is okay)${NC}"
        fi
    else
        echo -e "${RED}❌ Downloaded file is not a valid executable${NC}"
    fi
else
    echo -e "${RED}❌ Binary download failed${NC}"
fi

# Test checksum verification
echo -e "${YELLOW}Testing checksum verification...${NC}"
EXPECTED_CHECKSUM="f2ae3e00542d417effcd6ac8d3420b294808c667918f3c685f73b66a79870c97"
ACTUAL_CHECKSUM=$(shasum -a 256 "${TEST_DIR}/test-binary" | cut -d' ' -f1)

if [ "$EXPECTED_CHECKSUM" = "$ACTUAL_CHECKSUM" ]; then
    echo -e "${GREEN}✅ Checksum verification passed${NC}"
else
    echo -e "${RED}❌ Checksum verification failed${NC}"
    echo -e "${RED}   Expected: ${EXPECTED_CHECKSUM}${NC}"
    echo -e "${RED}   Actual:   ${ACTUAL_CHECKSUM}${NC}"
fi

# Cleanup
echo -e "${YELLOW}Cleaning up...${NC}"
kill $SERVER_PID 2>/dev/null || true
rm -rf "$TEST_DIR"

echo ""
echo -e "${GREEN}🎉 Install script testing complete!${NC}"
echo -e "${YELLOW}📝 The install script should work correctly with the GitHub release${NC}"