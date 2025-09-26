#!/bin/bash
# Update version across all files for Campfire release

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get version from command line or default to v0.1.0
VERSION=${1:-"v0.1.0"}
REPO=${2:-"that-in-rust/campfire-on-rust"}

echo -e "${BLUE}🔄 Updating Campfire to version ${VERSION}${NC}"

# Update Cargo.toml
echo -e "${YELLOW}Updating Cargo.toml...${NC}"
sed -i.bak "s/version = \".*\"/version = \"${VERSION#v}\"/" Cargo.toml

# Update install script
echo -e "${YELLOW}Updating install script...${NC}"
sed -i.bak "s/VERSION=\".*\"/VERSION=\"${VERSION}\"/" scripts/install.sh
sed -i.bak "s/REPO=\".*\"/REPO=\"${REPO}\"/" scripts/install.sh

# Update deploy script
echo -e "${YELLOW}Updating deploy script...${NC}"
sed -i.bak "s/GITHUB_REPO=\".*\"/GITHUB_REPO=\"${REPO}\"/" scripts/deploy-railway.sh

# Update Railway template
echo -e "${YELLOW}Updating Railway template...${NC}"
sed -i.bak "s/\"version\": \".*\"/\"version\": \"${VERSION#v}\"/" railway-template.json
sed -i.bak "s|\"repository\": \".*\"|\"repository\": \"https://github.com/${REPO}\"|" railway-template.json

# Update README links
echo -e "${YELLOW}Updating README links...${NC}"
sed -i.bak "s|your-org/campfire-rust|${REPO}|g" README.md
sed -i.bak "s|campfire-rust:v[0-9.]*|campfire-rust:${VERSION}|g" README.md

# Update deployment documentation
echo -e "${YELLOW}Updating deployment documentation...${NC}"
sed -i.bak "s|your-org/campfire-rust|${REPO}|g" docs/zero-friction-deployment.md
sed -i.bak "s|campfire-rust:v[0-9.]*|campfire-rust:${VERSION}|g" docs/zero-friction-deployment.md

# Clean up backup files
echo -e "${YELLOW}Cleaning up backup files...${NC}"
find . -name "*.bak" -delete

echo -e "${GREEN}✅ Version updated to ${VERSION}${NC}"
echo -e "${GREEN}✅ Repository updated to ${REPO}${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. ${YELLOW}Review changes: git diff${NC}"
echo -e "  2. ${YELLOW}Commit changes: git add . && git commit -m \"Release ${VERSION}\"${NC}"
echo -e "  3. ${YELLOW}Create tag: git tag ${VERSION}${NC}"
echo -e "  4. ${YELLOW}Push changes: git push origin main --tags${NC}"
echo -e "  5. ${YELLOW}GitHub Actions will build and release automatically${NC}"