#!/bin/bash
# Create GitHub Release v0.1.0 with available binaries
# This script creates a GitHub release and uploads the available binaries

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VERSION="v0.1.0"
REPO_OWNER="that-in-rust"
REPO_NAME="campfire-on-rust"

echo -e "${BLUE}🚀 Creating GitHub Release ${VERSION}${NC}"
echo ""

# Check if gh CLI is available
if ! command -v gh >/dev/null 2>&1; then
    echo -e "${RED}❌ GitHub CLI (gh) is required but not installed${NC}"
    echo -e "${YELLOW}💡 Install it from: https://cli.github.com/${NC}"
    exit 1
fi

# Check if we're in a git repository
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    echo -e "${RED}❌ Not in a git repository${NC}"
    exit 1
fi

# Check if we have release artifacts
if [ ! -d "release-artifacts" ] || [ ! -f "release-artifacts/campfire-on-rust-darwin-aarch64" ]; then
    echo -e "${RED}❌ Release artifacts not found${NC}"
    echo -e "${YELLOW}💡 Run ./scripts/build-release-binaries.sh first${NC}"
    exit 1
fi

# Create or update the tag
echo -e "${YELLOW}Creating git tag ${VERSION}...${NC}"
if git tag -l | grep -q "^${VERSION}$"; then
    echo -e "${YELLOW}⚠️  Tag ${VERSION} already exists, deleting and recreating...${NC}"
    git tag -d "${VERSION}" || true
    git push origin ":refs/tags/${VERSION}" || true
fi

git tag -a "${VERSION}" -m "Campfire ${VERSION} - Zero-Friction Chat Application"
git push origin "${VERSION}"

echo -e "${GREEN}✅ Git tag created and pushed${NC}"

# Create the GitHub release
echo -e "${YELLOW}Creating GitHub release...${NC}"

# Check if release already exists and delete it
if gh release view "${VERSION}" >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Release ${VERSION} already exists, deleting...${NC}"
    gh release delete "${VERSION}" --yes || true
fi

# Create the release with release notes
gh release create "${VERSION}" \
    --title "Campfire ${VERSION} - Zero-Friction Chat Application" \
    --notes-file "release-artifacts/RELEASE_NOTES_v0.1.0.md" \
    --draft=false \
    --prerelease=false

echo -e "${GREEN}✅ GitHub release created${NC}"

# Upload available binaries
echo -e "${YELLOW}Uploading release artifacts...${NC}"

cd release-artifacts

# Upload all available binaries
for binary in campfire-on-rust-*; do
    if [ -f "$binary" ] && [ "$binary" != "*.txt" ] && [ "$binary" != "*.md" ]; then
        echo -e "${YELLOW}Uploading ${binary}...${NC}"
        gh release upload "${VERSION}" "$binary" --clobber
        echo -e "${GREEN}✅ Uploaded ${binary}${NC}"
    fi
done

# Upload checksums
if [ -f "checksums.txt" ]; then
    echo -e "${YELLOW}Uploading checksums.txt...${NC}"
    gh release upload "${VERSION}" "checksums.txt" --clobber
    echo -e "${GREEN}✅ Uploaded checksums.txt${NC}"
fi

# Upload additional documentation
for doc in INSTALLATION.md RELEASE_NOTES.md; do
    if [ -f "$doc" ]; then
        echo -e "${YELLOW}Uploading ${doc}...${NC}"
        gh release upload "${VERSION}" "$doc" --clobber
        echo -e "${GREEN}✅ Uploaded ${doc}${NC}"
    fi
done

cd ..

echo ""
echo -e "${GREEN}🎉 GitHub Release ${VERSION} created successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Release Summary:${NC}"
echo -e "${YELLOW}   Release URL: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/tag/${VERSION}${NC}"
echo -e "${YELLOW}   Available binaries:${NC}"

cd release-artifacts
for binary in campfire-on-rust-*; do
    if [ -f "$binary" ] && [ "$binary" != "*.txt" ] && [ "$binary" != "*.md" ]; then
        size=$(ls -lh "$binary" | awk '{print $5}')
        echo -e "${YELLOW}     - ${binary} (${size})${NC}"
    fi
done
cd ..

echo ""
echo -e "${BLUE}📝 Next Steps:${NC}"
echo -e "${YELLOW}   1. The GitHub Actions workflow will build remaining binaries${NC}"
echo -e "${YELLOW}   2. Test the install script: curl -sSL https://raw.githubusercontent.com/${REPO_OWNER}/${REPO_NAME}/main/scripts/install.sh | bash${NC}"
echo -e "${YELLOW}   3. Update README with the release information${NC}"
echo -e "${YELLOW}   4. Announce the release!${NC}"