#!/bin/bash
# Validate the release is ready for deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Validating Campfire v0.1.0 Release${NC}"
echo ""

ERRORS=0

# Check if binaries exist
echo -e "${YELLOW}Checking release binaries...${NC}"
if [ -f "release-artifacts/campfire-on-rust-darwin-aarch64" ]; then
    echo -e "${GREEN}✅ ARM64 macOS binary exists${NC}"
else
    echo -e "${RED}❌ ARM64 macOS binary missing${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check checksums
echo -e "${YELLOW}Checking checksums...${NC}"
if [ -f "release-artifacts/checksums.txt" ]; then
    echo -e "${GREEN}✅ Checksums file exists${NC}"
    
    # Validate checksums
    cd release-artifacts
    if shasum -a 256 -c checksums.txt >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Checksums are valid${NC}"
    else
        echo -e "${RED}❌ Checksum validation failed${NC}"
        ERRORS=$((ERRORS + 1))
    fi
    cd ..
else
    echo -e "${RED}❌ Checksums file missing${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check release notes
echo -e "${YELLOW}Checking release notes...${NC}"
if [ -f "release-artifacts/RELEASE_NOTES_v0.1.0.md" ]; then
    echo -e "${GREEN}✅ Release notes exist${NC}"
else
    echo -e "${RED}❌ Release notes missing${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check GitHub Actions workflow
echo -e "${YELLOW}Checking GitHub Actions workflow...${NC}"
if [ -f ".github/workflows/release.yml" ]; then
    echo -e "${GREEN}✅ Release workflow exists${NC}"
else
    echo -e "${RED}❌ Release workflow missing${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check install script
echo -e "${YELLOW}Checking install script...${NC}"
if [ -f "scripts/install.sh" ]; then
    echo -e "${GREEN}✅ Install script exists${NC}"
    
    # Check if it has the correct repository URL
    if grep -q "that-in-rust/campfire-on-rust" scripts/install.sh; then
        echo -e "${GREEN}✅ Install script has correct repository URL${NC}"
    else
        echo -e "${RED}❌ Install script has incorrect repository URL${NC}"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo -e "${RED}❌ Install script missing${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check if code compiles
echo -e "${YELLOW}Checking if code compiles...${NC}"
if cargo check >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Code compiles successfully${NC}"
else
    echo -e "${RED}❌ Code compilation failed${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check README
echo -e "${YELLOW}Checking README...${NC}"
if [ -f "README.md" ]; then
    echo -e "${GREEN}✅ README exists${NC}"
else
    echo -e "${RED}❌ README missing${NC}"
    ERRORS=$((ERRORS + 1))
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}🎉 All validation checks passed!${NC}"
    echo -e "${GREEN}✅ Release is ready for deployment${NC}"
    echo ""
    echo -e "${BLUE}📋 Release Summary:${NC}"
    echo -e "${YELLOW}   Version: v0.1.0${NC}"
    echo -e "${YELLOW}   Available binaries: $(ls release-artifacts/campfire-on-rust-* 2>/dev/null | wc -l | tr -d ' ')${NC}"
    echo -e "${YELLOW}   Repository: that-in-rust/campfire-on-rust${NC}"
    echo ""
    echo -e "${BLUE}📝 Next Steps:${NC}"
    echo -e "${YELLOW}   1. Run: ./scripts/create-github-release.sh${NC}"
    echo -e "${YELLOW}   2. GitHub Actions will build remaining binaries${NC}"
    echo -e "${YELLOW}   3. Test the complete installation${NC}"
    exit 0
else
    echo -e "${RED}❌ ${ERRORS} validation error(s) found${NC}"
    echo -e "${RED}🚫 Release is NOT ready for deployment${NC}"
    exit 1
fi