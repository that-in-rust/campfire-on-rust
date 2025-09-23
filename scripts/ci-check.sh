#!/bin/bash
# CI check script - runs the same checks as our CI pipeline
# Use this to verify your changes before pushing

set -e

echo "🚀 Running CI checks locally..."

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2 passed${NC}"
    else
        echo -e "${RED}❌ $2 failed${NC}"
        exit 1
    fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found. Run this script from the project root.${NC}"
    exit 1
fi

echo -e "${YELLOW}📋 Running format check...${NC}"
cargo fmt -- --check
print_status $? "Format check"

echo -e "${YELLOW}🔍 Running clippy lints...${NC}"
cargo clippy --all-targets --all-features -- -D warnings
print_status $? "Clippy lints"

echo -e "${YELLOW}🧪 Running tests...${NC}"
cargo test --all-features
print_status $? "Tests"

echo -e "${YELLOW}🔒 Running security audit...${NC}"
cargo audit
print_status $? "Security audit"

echo -e "${YELLOW}🏗️  Building release binary...${NC}"
cargo build --release
print_status $? "Release build"

echo -e "${YELLOW}📊 Checking test coverage...${NC}"
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Stdout --fail-under 80
    print_status $? "Test coverage (≥80%)"
else
    echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed, skipping coverage check${NC}"
    echo "Install with: cargo install cargo-tarpaulin"
fi

echo -e "${GREEN}🎉 All CI checks passed! Your code is ready for submission.${NC}"