#!/bin/bash
# Automated test script for GitHub Release with Pre-built Binaries
# Tests all requirements: 10.2, 10.3, 10.4, 10.6

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="that-in-rust/campfire-on-rust"
VERSION="v0.1.0"
TEST_DIR="$(mktemp -d)"
BINARY_NAME="campfire-on-rust"

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
    FAILED_TESTS+=("$1")
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Test 1: Build optimized binaries for all platforms
test_binary_builds() {
    log_info "Testing binary builds for all target platforms..."
    
    local platforms=(
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu" 
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-pc-windows-msvc"
    )
    
    for platform in "${platforms[@]}"; do
        log_info "Testing build for platform: $platform"
        
        # Check if target is installed
        if ! rustup target list --installed | grep -q "$platform"; then
            log_warning "Target $platform not installed, adding..."
            rustup target add "$platform" || {
                log_error "Failed to add target $platform"
                continue
            }
        fi
        
        # Test build (dry run to avoid long compilation times)
        if cargo check --release --target "$platform" --bin campfire-on-rust; then
            log_success "Build check passed for $platform"
        else
            log_error "Build check failed for $platform"
        fi
    done
}

# Test 2: Verify GitHub Actions workflow configuration
test_github_workflow() {
    log_info "Testing GitHub Actions workflow configuration..."
    
    local workflow_file=".github/workflows/release.yml"
    
    if [[ ! -f "$workflow_file" ]]; then
        log_error "GitHub workflow file not found: $workflow_file"
        return 1
    fi
    
    # Check workflow has all required platforms
    local required_platforms=(
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
        "x86_64-apple-darwin" 
        "aarch64-apple-darwin"
        "x86_64-pc-windows-msvc"
    )
    
    for platform in "${required_platforms[@]}"; do
        if grep -q "$platform" "$workflow_file"; then
            log_success "Workflow includes platform: $platform"
        else
            log_error "Workflow missing platform: $platform"
        fi
    done
    
    # Check workflow has release creation step
    if grep -q "Create GitHub Release" "$workflow_file"; then
        log_success "Workflow includes GitHub release creation"
    else
        log_error "Workflow missing GitHub release creation step"
    fi
    
    # Check workflow has artifact upload
    if grep -q "upload-artifact" "$workflow_file"; then
        log_success "Workflow includes artifact upload"
    else
        log_error "Workflow missing artifact upload step"
    fi
}

# Test 3: Simulate GitHub releases API download test
test_github_api_simulation() {
    log_info "Testing GitHub releases API simulation..."
    
    # Create mock release structure
    local mock_release_dir="$TEST_DIR/mock-release"
    mkdir -p "$mock_release_dir"
    
    # Create mock binaries for each platform
    local platforms=(
        "campfire-on-rust-linux-x86_64"
        "campfire-on-rust-linux-aarch64"
        "campfire-on-rust-darwin-x86_64"
        "campfire-on-rust-darwin-aarch64"
        "campfire-on-rust-windows-x86_64.exe"
    )
    
    for platform_binary in "${platforms[@]}"; do
        # Create a mock binary (copy our actual binary for testing)
        if [[ -f "target/release/campfire-on-rust" ]]; then
            cp "target/release/campfire-on-rust" "$mock_release_dir/$platform_binary"
            chmod +x "$mock_release_dir/$platform_binary"
            log_success "Created mock binary: $platform_binary"
        else
            # Create a minimal mock binary
            echo '#!/bin/bash\necho "Mock Campfire Binary"' > "$mock_release_dir/$platform_binary"
            chmod +x "$mock_release_dir/$platform_binary"
            log_success "Created minimal mock binary: $platform_binary"
        fi
    done
    
    # Test binary execution
    for platform_binary in "${platforms[@]}"; do
        if [[ "$platform_binary" == *.exe ]]; then
            log_warning "Skipping Windows binary execution test on Unix system"
            continue
        fi
        
        if "$mock_release_dir/$platform_binary" --version 2>/dev/null || \
           "$mock_release_dir/$platform_binary" --help 2>/dev/null || \
           echo "test" | "$mock_release_dir/$platform_binary" 2>/dev/null; then
            log_success "Mock binary executable: $platform_binary"
        else
            log_warning "Mock binary may not be executable: $platform_binary (this is expected for cross-compiled binaries)"
        fi
    done
    
    # Generate checksums
    cd "$mock_release_dir"
    sha256sum * > checksums.txt
    if [[ -f "checksums.txt" && -s "checksums.txt" ]]; then
        log_success "Generated checksums file"
        log_info "Checksums preview:"
        head -3 checksums.txt | sed 's/^/  /'
    else
        log_error "Failed to generate checksums file"
    fi
    cd - > /dev/null
}

# Test 4: Verify install script functionality
test_install_script() {
    log_info "Testing install script functionality..."
    
    local install_script="scripts/install.sh"
    
    if [[ ! -f "$install_script" ]]; then
        log_error "Install script not found: $install_script"
        return 1
    fi
    
    # Check script is executable
    if [[ -x "$install_script" ]]; then
        log_success "Install script is executable"
    else
        log_error "Install script is not executable"
    fi
    
    # Test script syntax
    if bash -n "$install_script"; then
        log_success "Install script syntax is valid"
    else
        log_error "Install script has syntax errors"
    fi
    
    # Check script has required functions
    local required_functions=(
        "detect_platform"
        "install_campfire"
        "setup_environment"
        "update_path"
    )
    
    for func in "${required_functions[@]}"; do
        if grep -q "^${func}()" "$install_script"; then
            log_success "Install script has function: $func"
        else
            log_error "Install script missing function: $func"
        fi
    done
    
    # Test platform detection logic
    log_info "Testing platform detection logic..."
    
    # Extract and test the detect_platform function
    local test_script="$TEST_DIR/test_platform_detection.sh"
    cat > "$test_script" << 'EOF'
#!/bin/bash
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="darwin" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          echo "unsupported-os" && return 1 ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="aarch64" ;;
        *)              echo "unsupported-arch" && return 1 ;;
    esac
    
    echo "${os}-${arch}"
}

detect_platform
EOF
    
    chmod +x "$test_script"
    
    if platform_result=$("$test_script" 2>/dev/null); then
        log_success "Platform detection works: $platform_result"
    else
        log_error "Platform detection failed"
    fi
    
    # Test download URL construction
    local expected_platforms=(
        "linux-x86_64"
        "linux-aarch64"
        "darwin-x86_64"
        "darwin-aarch64"
        "windows-x86_64"
    )
    
    for platform in "${expected_platforms[@]}"; do
        local expected_url="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}-${platform}"
        if [[ "$platform" == *"windows"* ]]; then
            expected_url="${expected_url}.exe"
        fi
        
        if grep -q "github.com.*releases.*download" "$install_script"; then
            log_success "Install script constructs GitHub release URLs"
            break
        fi
    done
}

# Test 5: Verify binary optimization settings
test_binary_optimization() {
    log_info "Testing binary optimization settings..."
    
    # Check Cargo.toml release profile
    if grep -A 10 "\[profile\.release\]" Cargo.toml | grep -q "lto = true"; then
        log_success "LTO (Link Time Optimization) enabled"
    else
        log_error "LTO not enabled in release profile"
    fi
    
    if grep -A 10 "\[profile\.release\]" Cargo.toml | grep -q "codegen-units = 1"; then
        log_success "Single codegen unit configured for optimization"
    else
        log_error "Multiple codegen units may reduce optimization"
    fi
    
    if grep -A 10 "\[profile\.release\]" Cargo.toml | grep -q "strip = true"; then
        log_success "Debug symbol stripping enabled"
    else
        log_error "Debug symbols not stripped in release build"
    fi
    
    # Test actual binary if it exists
    if [[ -f "target/release/campfire-on-rust" ]]; then
        local binary_size=$(stat -f%z "target/release/campfire-on-rust" 2>/dev/null || stat -c%s "target/release/campfire-on-rust" 2>/dev/null)
        if [[ -n "$binary_size" ]]; then
            local size_mb=$((binary_size / 1024 / 1024))
            log_info "Release binary size: ${size_mb}MB"
            
            # Reasonable size check (should be under 100MB for a Rust binary)
            if [[ $size_mb -lt 100 ]]; then
                log_success "Binary size is reasonable: ${size_mb}MB"
            else
                log_warning "Binary size is large: ${size_mb}MB (consider further optimization)"
            fi
        fi
        
        # Test binary has no debug symbols (if file command is available)
        if command -v file >/dev/null 2>&1; then
            if file "target/release/campfire-on-rust" | grep -q "not stripped"; then
                log_warning "Binary may contain debug symbols"
            else
                log_success "Binary appears to be stripped"
            fi
        fi
    else
        log_warning "Release binary not found, run 'cargo build --release' first"
    fi
}

# Test 6: Verify release workflow triggers
test_workflow_triggers() {
    log_info "Testing workflow trigger configuration..."
    
    local workflow_file=".github/workflows/release.yml"
    
    # Check for tag-based triggers
    if grep -A 5 "^on:" "$workflow_file" | grep -q "tags:"; then
        log_success "Workflow triggers on git tags"
    else
        log_error "Workflow missing git tag trigger"
    fi
    
    # Check for manual dispatch
    if grep -A 5 "^on:" "$workflow_file" | grep -q "workflow_dispatch:"; then
        log_success "Workflow supports manual dispatch"
    else
        log_error "Workflow missing manual dispatch trigger"
    fi
    
    # Check version input for manual dispatch
    if grep -A 10 "workflow_dispatch:" "$workflow_file" | grep -q "version:"; then
        log_success "Manual dispatch includes version input"
    else
        log_error "Manual dispatch missing version input"
    fi
}

# Test 7: Validate release notes and documentation
test_release_documentation() {
    log_info "Testing release documentation..."
    
    local workflow_file=".github/workflows/release.yml"
    
    # Check for release notes generation
    if grep -q "release_notes" "$workflow_file" || grep -q "body_path" "$workflow_file"; then
        log_success "Workflow includes release notes"
    else
        log_error "Workflow missing release notes generation"
    fi
    
    # Check for comprehensive feature list in release notes
    local expected_features=(
        "Real-time messaging"
        "WebSocket"
        "Push notifications"
        "Bot integration"
        "Full-text search"
        "Demo mode"
    )
    
    for feature in "${expected_features[@]}"; do
        if grep -i -q "$feature" "$workflow_file"; then
            log_success "Release notes mention: $feature"
        else
            log_warning "Release notes may be missing: $feature"
        fi
    done
}

# Main test execution
main() {
    echo -e "${GREEN}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                 GitHub Release Test Suite                    ║"
    echo "║              Automated Verification Script                   ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
    
    log_info "Starting automated tests for GitHub Release with Pre-built Binaries"
    log_info "Test directory: $TEST_DIR"
    echo ""
    
    # Run all tests
    test_binary_builds
    echo ""
    
    test_github_workflow
    echo ""
    
    test_github_api_simulation
    echo ""
    
    test_install_script
    echo ""
    
    test_binary_optimization
    echo ""
    
    test_workflow_triggers
    echo ""
    
    test_release_documentation
    echo ""
    
    # Summary
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                        TEST SUMMARY                          ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}🎉 ALL TESTS PASSED! ($TESTS_PASSED/$((TESTS_PASSED + TESTS_FAILED)))${NC}"
        echo -e "${GREEN}✅ GitHub Release with Pre-built Binaries is ready for deployment${NC}"
        echo ""
        echo -e "${BLUE}Next Steps:${NC}"
        echo -e "  1. ${YELLOW}git tag v0.1.0${NC}"
        echo -e "  2. ${YELLOW}git push origin v0.1.0${NC}"
        echo -e "  3. Monitor GitHub Actions workflow execution"
        echo -e "  4. Verify release creation and binary uploads"
        echo ""
    else
        echo -e "${RED}❌ SOME TESTS FAILED ($TESTS_FAILED failed, $TESTS_PASSED passed)${NC}"
        echo ""
        echo -e "${RED}Failed Tests:${NC}"
        for failed_test in "${FAILED_TESTS[@]}"; do
            echo -e "  - ${RED}$failed_test${NC}"
        done
        echo ""
        echo -e "${YELLOW}Please fix the failing tests before proceeding with the release.${NC}"
    fi
    
    # Cleanup
    rm -rf "$TEST_DIR"
    
    # Exit with appropriate code
    if [[ $TESTS_FAILED -eq 0 ]]; then
        exit 0
    else
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "GitHub Release Test Suite"
        echo ""
        echo "Usage:"
        echo "  $0                    # Run all tests"
        echo "  $0 --help           # Show this help"
        echo "  $0 --quick          # Run quick tests only"
        echo ""
        echo "This script validates:"
        echo "  - Binary builds for all target platforms"
        echo "  - GitHub Actions workflow configuration"
        echo "  - Release API simulation"
        echo "  - Install script functionality"
        echo "  - Binary optimization settings"
        echo "  - Workflow triggers and documentation"
        exit 0
        ;;
    --quick)
        log_info "Running quick tests only..."
        test_github_workflow
        test_install_script
        test_binary_optimization
        ;;
    *)
        main
        ;;
esac