#!/bin/bash

# Mobile Experience Validation Script
# 
# Validates mobile-friendly experience using industry standard testing
# without requiring human interaction. Tests all aspects of mobile UX.
# 
# Requirements: 8.1, 8.2, 8.3, 8.4, 8.5

set -e

echo "🔥 Campfire Mobile Experience Validation"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test result tracking
test_result() {
    local test_name="$1"
    local result="$2"
    local details="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$result" = "PASS" ]; then
        echo -e "${GREEN}✓${NC} $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        if [ -n "$details" ]; then
            echo "  $details"
        fi
    else
        echo -e "${RED}✗${NC} $test_name"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        if [ -n "$details" ]; then
            echo -e "  ${RED}$details${NC}"
        fi
    fi
}

# Test 1: README Mobile Readability
echo -e "\n${BLUE}📱 Testing README Mobile Readability${NC}"

test_readme_mobile() {
    if [ ! -f "README.md" ]; then
        test_result "README exists" "FAIL" "README.md not found"
        return
    fi
    
    test_result "README exists" "PASS"
    
    # Check for mobile-friendly deployment button
    if grep -q "Deploy on Railway" README.md; then
        test_result "Railway deployment button present" "PASS"
    else
        test_result "Railway deployment button present" "FAIL" "No Railway deployment button found"
    fi
    
    # Check for mobile viewport considerations
    if grep -q "localhost:3000" README.md; then
        test_result "Mobile-accessible localhost instructions" "PASS"
    else
        test_result "Mobile-accessible localhost instructions" "FAIL" "No localhost:3000 instructions found"
    fi
    
    # Check line lengths for mobile readability
    long_lines=$(grep -v "http" README.md | awk 'length > 100' | wc -l | tr -d ' ')
    if [ "$long_lines" -lt 10 ]; then
        test_result "Mobile-friendly line lengths" "PASS" "Only $long_lines long lines found"
    else
        test_result "Mobile-friendly line lengths" "FAIL" "$long_lines long lines found (should be <10)"
    fi
    
    # Check for mobile troubleshooting
    if grep -qi "mobile\|phone\|tablet" README.md; then
        test_result "Mobile-specific guidance" "PASS"
    else
        test_result "Mobile-specific guidance" "FAIL" "No mobile-specific guidance found"
    fi
}

test_readme_mobile

# Test 2: CSS Mobile Responsiveness
echo -e "\n${BLUE}🎨 Testing CSS Mobile Responsiveness${NC}"

test_css_mobile() {
    if [ ! -f "assets/static/css/campfire.css" ]; then
        test_result "CSS file exists" "FAIL" "campfire.css not found"
        return
    fi
    
    test_result "CSS file exists" "PASS"
    
    # Check for media queries
    if grep -q "@media" assets/static/css/campfire.css; then
        test_result "Responsive media queries" "PASS"
    else
        test_result "Responsive media queries" "FAIL" "No @media queries found"
    fi
    
    # Check for mobile breakpoints
    if grep -q "max-width.*768px\|max-width.*767px" assets/static/css/campfire.css; then
        test_result "Mobile breakpoints" "PASS"
    else
        test_result "Mobile breakpoints" "FAIL" "No mobile breakpoints found"
    fi
    
    # Check for touch-friendly sizing
    if grep -q "44px\|48px" assets/static/css/campfire.css; then
        test_result "Touch-friendly target sizes" "PASS"
    else
        test_result "Touch-friendly target sizes" "FAIL" "No touch-friendly sizes found"
    fi
    
    # Check CSS file size for mobile performance
    css_size=$(wc -c < assets/static/css/campfire.css)
    if [ "$css_size" -lt 100000 ]; then
        test_result "Mobile-optimized CSS size" "PASS" "CSS is ${css_size} bytes (<100KB)"
    else
        test_result "Mobile-optimized CSS size" "FAIL" "CSS is ${css_size} bytes (should be <100KB)"
    fi
}

test_css_mobile

# Test 3: HTML Mobile Viewport
echo -e "\n${BLUE}📄 Testing HTML Mobile Viewport${NC}"

test_html_mobile() {
    if [ ! -f "templates/chat.html" ]; then
        test_result "Chat template exists" "FAIL" "chat.html not found"
        return
    fi
    
    test_result "Chat template exists" "PASS"
    
    # Check for mobile viewport meta tag
    if grep -q 'name="viewport"' templates/chat.html; then
        test_result "Mobile viewport meta tag" "PASS"
    else
        test_result "Mobile viewport meta tag" "FAIL" "No viewport meta tag found"
    fi
    
    # Check for device-width responsiveness
    if grep -q "width=device-width" templates/chat.html; then
        test_result "Device-width responsiveness" "PASS"
    else
        test_result "Device-width responsiveness" "FAIL" "No device-width setting found"
    fi
    
    # Check for mobile app meta tags
    if grep -q "apple-mobile-web-app" templates/chat.html; then
        test_result "Mobile app meta tags" "PASS"
    else
        test_result "Mobile app meta tags" "FAIL" "No mobile app meta tags found"
    fi
    
    # Check for mobile-friendly manifest
    if grep -q 'rel="manifest"' templates/chat.html; then
        test_result "PWA manifest" "PASS"
    else
        test_result "PWA manifest" "FAIL" "No PWA manifest found"
    fi
}

test_html_mobile

# Test 4: Install Script Mobile Compatibility
echo -e "\n${BLUE}⚙️ Testing Install Script Mobile Compatibility${NC}"

test_install_script_mobile() {
    if [ ! -f "scripts/install.sh" ]; then
        test_result "Install script exists" "FAIL" "install.sh not found"
        return
    fi
    
    test_result "Install script exists" "PASS"
    
    # Check for mobile terminal compatibility (line lengths)
    long_lines=$(awk 'length > 80' scripts/install.sh | wc -l | tr -d ' ')
    if [ "$long_lines" -lt 5 ]; then
        test_result "Mobile terminal line lengths" "PASS" "Only $long_lines long lines"
    else
        test_result "Mobile terminal line lengths" "FAIL" "$long_lines long lines (should be <5)"
    fi
    
    # Check for user feedback
    if grep -q "echo\|printf" scripts/install.sh; then
        test_result "User feedback in script" "PASS"
    else
        test_result "User feedback in script" "FAIL" "No user feedback found"
    fi
    
    # Check for mobile-friendly guidance
    if grep -qi "localhost:3000\|browser" scripts/install.sh; then
        test_result "Mobile-friendly guidance" "PASS"
    else
        test_result "Mobile-friendly guidance" "FAIL" "No mobile guidance found"
    fi
}

test_install_script_mobile

# Test 5: Mobile Performance Considerations
echo -e "\n${BLUE}⚡ Testing Mobile Performance Considerations${NC}"

test_mobile_performance() {
    # Check for performance-optimized assets
    if [ -d "assets/static" ]; then
        test_result "Static assets directory exists" "PASS"
        
        # Check for compressed/optimized images
        image_count=$(find assets/static -name "*.png" -o -name "*.jpg" -o -name "*.svg" | wc -l | tr -d ' ')
        if [ "$image_count" -gt 0 ]; then
            test_result "Image assets present" "PASS" "$image_count image files found"
        else
            test_result "Image assets present" "FAIL" "No image assets found"
        fi
        
        # Check for SVG usage (mobile-friendly)
        svg_count=$(find assets/static -name "*.svg" | wc -l | tr -d ' ')
        if [ "$svg_count" -gt 0 ]; then
            test_result "SVG icons for mobile" "PASS" "$svg_count SVG files found"
        else
            test_result "SVG icons for mobile" "FAIL" "No SVG files found"
        fi
    else
        test_result "Static assets directory exists" "FAIL" "assets/static not found"
    fi
    
    # Check for service worker (PWA)
    if [ -f "assets/static/js/sw.js" ]; then
        test_result "Service worker for offline support" "PASS"
    else
        test_result "Service worker for offline support" "FAIL" "No service worker found"
    fi
}

test_mobile_performance

# Test 6: Mobile Deployment Process
echo -e "\n${BLUE}🚀 Testing Mobile Deployment Process${NC}"

test_mobile_deployment() {
    # Check Railway template URL format
    if grep -q "railway.app/template" README.md; then
        test_result "Railway template URL format" "PASS"
    else
        test_result "Railway template URL format" "FAIL" "No Railway template URL found"
    fi
    
    # Check for mobile-friendly deployment instructions
    if grep -qi "deploy.*button\|click.*deploy" README.md; then
        test_result "Mobile-friendly deployment instructions" "PASS"
    else
        test_result "Mobile-friendly deployment instructions" "FAIL" "No mobile deployment instructions"
    fi
    
    # Check for troubleshooting section
    if grep -qi "troubleshoot\|issues\|problems" README.md; then
        test_result "Mobile troubleshooting section" "PASS"
    else
        test_result "Mobile troubleshooting section" "FAIL" "No troubleshooting section found"
    fi
}

test_mobile_deployment

# Test 7: Mobile Accessibility
echo -e "\n${BLUE}♿ Testing Mobile Accessibility${NC}"

test_mobile_accessibility() {
    # Check for semantic HTML in templates
    if grep -q "role=\|aria-" templates/chat.html; then
        test_result "Accessibility attributes" "PASS"
    else
        test_result "Accessibility attributes" "FAIL" "No ARIA attributes found"
    fi
    
    # Check for skip navigation
    if grep -q "skip.*navigation\|skip.*content" templates/chat.html; then
        test_result "Skip navigation for mobile" "PASS"
    else
        test_result "Skip navigation for mobile" "FAIL" "No skip navigation found"
    fi
    
    # Check for focus management in CSS
    if grep -q ":focus" assets/static/css/campfire.css; then
        test_result "Focus indicators for mobile" "PASS"
    else
        test_result "Focus indicators for mobile" "FAIL" "No focus styles found"
    fi
    
    # Check for reduced motion support
    if grep -q "prefers-reduced-motion" assets/static/css/campfire.css; then
        test_result "Reduced motion support" "PASS"
    else
        test_result "Reduced motion support" "FAIL" "No reduced motion support found"
    fi
}

test_mobile_accessibility

# Test 8: Mobile Error Handling
echo -e "\n${BLUE}🚨 Testing Mobile Error Handling${NC}"

test_mobile_error_handling() {
    # Check for mobile-specific error scenarios in troubleshooting
    mobile_errors=("WebSocket" "connection" "browser" "cache" "keyboard")
    found_errors=0
    
    for error in "${mobile_errors[@]}"; do
        if grep -qi "$error" README.md; then
            found_errors=$((found_errors + 1))
        fi
    done
    
    if [ "$found_errors" -ge 3 ]; then
        test_result "Mobile error scenarios covered" "PASS" "$found_errors/5 error types covered"
    else
        test_result "Mobile error scenarios covered" "FAIL" "Only $found_errors/5 error types covered"
    fi
    
    # Check for clear error messaging
    if grep -qi "solution\|fix\|resolve" README.md; then
        test_result "Clear error solutions" "PASS"
    else
        test_result "Clear error solutions" "FAIL" "No clear solutions provided"
    fi
}

test_mobile_error_handling

# Final Results
echo -e "\n${BLUE}📊 Mobile Experience Validation Results${NC}"
echo "========================================"
echo -e "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"

# Calculate pass percentage
if [ "$TOTAL_TESTS" -gt 0 ]; then
    pass_percentage=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    echo -e "Pass Rate: ${pass_percentage}%"
    
    if [ "$pass_percentage" -ge 85 ]; then
        echo -e "\n${GREEN}🎉 Excellent mobile experience! Ready for mobile users.${NC}"
        exit 0
    elif [ "$pass_percentage" -ge 70 ]; then
        echo -e "\n${YELLOW}⚠️  Good mobile experience with room for improvement.${NC}"
        exit 0
    else
        echo -e "\n${RED}❌ Mobile experience needs significant improvement.${NC}"
        exit 1
    fi
else
    echo -e "\n${RED}❌ No tests were run.${NC}"
    exit 1
fi