#!/bin/bash
# Build release binaries for all supported platforms
# This script builds what it can locally and prepares for GitHub Actions to build the rest

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔥 Building Campfire v0.1.0 Release Binaries${NC}"
echo ""

# Create release directory
mkdir -p release-artifacts
cd release-artifacts

# Clean previous builds
rm -f campfire-on-rust-* checksums.txt

echo -e "${YELLOW}Building native ARM64 macOS binary...${NC}"
cd ..
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/campfire-on-rust release-artifacts/campfire-on-rust-darwin-aarch64
echo -e "${GREEN}✅ ARM64 macOS binary built${NC}"

# Try to build x86_64 macOS if possible (might fail due to OpenSSL cross-compilation)
echo -e "${YELLOW}Attempting x86_64 macOS binary...${NC}"
if cargo build --release --target x86_64-apple-darwin 2>/dev/null; then
    cp target/x86_64-apple-darwin/release/campfire-on-rust release-artifacts/campfire-on-rust-darwin-x86_64
    echo -e "${GREEN}✅ x86_64 macOS binary built${NC}"
else
    echo -e "${YELLOW}⚠️  x86_64 macOS binary build failed (will be built by GitHub Actions)${NC}"
fi

# Try to build Linux binaries if cross is available
if command -v cross >/dev/null 2>&1 && (command -v docker >/dev/null 2>&1 || command -v podman >/dev/null 2>&1); then
    echo -e "${YELLOW}Building Linux x86_64 binary with cross...${NC}"
    if cross build --release --target x86_64-unknown-linux-gnu; then
        cp target/x86_64-unknown-linux-gnu/release/campfire-on-rust release-artifacts/campfire-on-rust-linux-x86_64
        echo -e "${GREEN}✅ Linux x86_64 binary built${NC}"
    else
        echo -e "${YELLOW}⚠️  Linux x86_64 binary build failed (will be built by GitHub Actions)${NC}"
    fi
    
    echo -e "${YELLOW}Building Linux ARM64 binary with cross...${NC}"
    if cross build --release --target aarch64-unknown-linux-gnu; then
        cp target/aarch64-unknown-linux-gnu/release/campfire-on-rust release-artifacts/campfire-on-rust-linux-aarch64
        echo -e "${GREEN}✅ Linux ARM64 binary built${NC}"
    else
        echo -e "${YELLOW}⚠️  Linux ARM64 binary build failed (will be built by GitHub Actions)${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Cross-compilation tools not available (Linux binaries will be built by GitHub Actions)${NC}"
fi

# Windows builds require special setup, skip for now
echo -e "${YELLOW}⚠️  Windows binary will be built by GitHub Actions${NC}"

# Generate checksums for available binaries
cd release-artifacts
echo -e "${YELLOW}Generating checksums...${NC}"
if ls campfire-on-rust-* >/dev/null 2>&1; then
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum campfire-on-rust-* > checksums.txt
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 campfire-on-rust-* > checksums.txt
    else
        echo -e "${RED}❌ No SHA256 tool available${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Checksums generated${NC}"
    echo ""
    echo -e "${BLUE}Available binaries:${NC}"
    ls -la campfire-on-rust-*
    echo ""
    echo -e "${BLUE}Checksums:${NC}"
    cat checksums.txt
else
    echo -e "${RED}❌ No binaries were built successfully${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Local binary build complete!${NC}"
echo -e "${YELLOW}📝 Next steps:${NC}"
echo -e "${YELLOW}   1. Commit and push changes${NC}"
echo -e "${YELLOW}   2. Create GitHub release tag: git tag v0.1.0${NC}"
echo -e "${YELLOW}   3. Push tag: git push origin v0.1.0${NC}"
echo -e "${YELLOW}   4. GitHub Actions will build remaining binaries and create the release${NC}"