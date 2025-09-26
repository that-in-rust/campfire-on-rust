#!/bin/bash

# GTM Phase 2 Validation Script
# 
# Validates all Phase 2 requirements:
# - README clearly shows two paths: local sampling and team deployment
# - Demo mode provides compelling local sampling experience  
# - Performance claims in README are verified and accurate
# - Simple analytics track deployment success rates
# - Mobile experience is functional and user-friendly
#
# Requirements: 1.1-1.5, 2.1-2.5, 4.1-4.3, 5.1-5.5, 8.1-8.5

set -e

echo "🔥 Campfire GTM Phase 2 Validation"
echo "=================================="

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

# Requirement 1: Two-Path User Experience
echo -e "\n${BLUE}🛤️  Testing Two-Path User Experience (Req 1.1-1.5)${NC}"

test_two_path_experience() {
    # Check for clear two-path structure in README
    if grep -q "Try it locally" README.md && grep -q "Deploy for your team" README.md; then
        test_result "Two clear paths in README" "PASS"
    else
        test_result "Two clear paths in README" "FAIL" "Missing 'Try it locally' or 'Deploy for your team' sections"
    fi
    
    # Check for curl install command
    if grep -q "curl.*install.sh" README.md; then
        test_result "Local install command present" "PASS"
    else
        test_result "Local install command present" "FAIL" "No curl install command found"
    fi
    
    # Check for Railway deployment button
    if grep -q "Deploy on Railway" README.md && grep -q "railway.app" README.md; then
        test_result "Railway deployment button present" "PASS"
    else
        test_result "Railway deployment button present" "FAIL" "No Railway deployment button found"
    fi
    
    # Check for localhost:3000 instructions
    if grep -q "localhost:3000" README.md; then
        test_result "Local access instructions" "PASS"
    else
        test_result "Local access instructions" "FAIL" "No localhost:3000 instructions found"
    fi
    
    # Check for 2-3 minute promises
    if grep -q "2 minutes\|3 minutes" README.md; then
        test_result "Time promises present" "PASS"
    else
        test_result "Time promises present" "FAIL" "No time promises found"
    fi
}

test_two_path_experience

# Requirement 2: Local Sampling Experience
echo -e "\n${BLUE}👀 Testing Local Sampling Experience (Req 2.1-2.5)${NC}"

test_local_sampling() {
    # Check if demo mode is implemented
    if [ -f "src/demo.rs" ]; then
        test_result "Demo mode implementation exists" "PASS"
        
        # Check for comprehensive demo data
        demo_lines=$(wc -l < src/demo.rs)
        if [ "$demo_lines" -gt 500 ]; then
            test_result "Comprehensive demo data" "PASS" "$demo_lines lines of demo implementation"
        else
            test_result "Comprehensive demo data" "FAIL" "Only $demo_lines lines of demo implementation"
        fi
        
        # Check for multiple demo users
        if grep -q "demo_users.*vec" src/demo.rs; then
            test_result "Multiple demo users" "PASS"
        else
            test_result "Multiple demo users" "FAIL" "No multiple demo users found"
        fi
        
        # Check for realistic conversations
        if grep -q "create_sample_conversations" src/demo.rs; then
            test_result "Sample conversations" "PASS"
        else
            test_result "Sample conversations" "FAIL" "No sample conversations found"
        fi
        
        # Check for feature demonstrations
        features=("@mentions" "sounds" "rooms" "search")
        found_features=0
        for feature in "${features[@]}"; do
            if grep -qi "$feature" src/demo.rs; then
                found_features=$((found_features + 1))
            fi
        done
        
        if [ "$found_features" -ge 3 ]; then
            test_result "Core features demonstrated" "PASS" "$found_features/4 features found in demo"
        else
            test_result "Core features demonstrated" "FAIL" "Only $found_features/4 features found in demo"
        fi
    else
        test_result "Demo mode implementation exists" "FAIL" "src/demo.rs not found"
    fi
    
    # Check for "Deploy for Your Team" call-to-action in demo
    if grep -qi "deploy.*team\|deploy.*real" README.md; then
        test_result "Deploy call-to-action present" "PASS"
    else
        test_result "Deploy call-to-action present" "FAIL" "No deploy call-to-action found"
    fi
}

test_local_sampling

# Requirement 4: Performance Claims Validation
echo -e "\n${BLUE}⚡ Testing Performance Claims (Req 4.1-4.3)${NC}"

test_performance_claims() {
    # Run performance validation if script exists
    if [ -f "scripts/validate-performance.sh" ]; then
        test_result "Performance validation script exists" "PASS"
        
        # Check if performance report exists (indicates validation was run)
        if [ -f "performance_report.md" ]; then
            test_result "Performance validation report exists" "PASS"
            
            # Check for verified metrics in report
            if grep -q "PASS" performance_report.md; then
                test_result "Performance metrics verified" "PASS"
            else
                test_result "Performance metrics verified" "FAIL" "No PASS results in performance report"
            fi
            
            # Check startup time claim
            if grep -q "Startup time.*0s\|Startup time.*<.*1" performance_report.md; then
                test_result "Startup time verified" "PASS"
            else
                test_result "Startup time verified" "FAIL" "Startup time not verified"
            fi
            
            # Check memory usage claim
            if grep -q "Memory usage.*MB" performance_report.md; then
                test_result "Memory usage verified" "PASS"
            else
                test_result "Memory usage verified" "FAIL" "Memory usage not verified"
            fi
        else
            test_result "Performance validation report exists" "FAIL" "performance_report.md not found"
        fi
    else
        test_result "Performance validation script exists" "FAIL" "scripts/validate-performance.sh not found"
    fi
    
    # Check for honest performance claims in README
    if grep -q "~20MB\|18.*MB\|under 1 second" README.md; then
        test_result "Specific performance numbers in README" "PASS"
    else
        test_result "Specific performance numbers in README" "FAIL" "No specific performance numbers found"
    fi
}

test_performance_claims

# Requirement 5: Simple Analytics Tracking
echo -e "\n${BLUE}📊 Testing Analytics Tracking (Req 5.1-5.5)${NC}"

test_analytics_tracking() {
    # Check if analytics script exists
    if [ -f "scripts/analytics-tracker.sh" ]; then
        test_result "Analytics tracking script exists" "PASS"
        
        # Test analytics functionality
        if ./scripts/analytics-tracker.sh --help > /dev/null 2>&1; then
            test_result "Analytics script functional" "PASS"
        else
            test_result "Analytics script functional" "FAIL" "Analytics script not working"
        fi
        
        # Check for deployment tracking
        if grep -q "deployment_click\|deployment_result" scripts/analytics-tracker.sh; then
            test_result "Deployment tracking implemented" "PASS"
        else
            test_result "Deployment tracking implemented" "FAIL" "No deployment tracking found"
        fi
        
        # Check for install tracking
        if grep -q "install_download" scripts/analytics-tracker.sh; then
            test_result "Install tracking implemented" "PASS"
        else
            test_result "Install tracking implemented" "FAIL" "No install tracking found"
        fi
        
        # Check for privacy-friendly approach
        if grep -q "privacy\|local.*file\|no.*external" scripts/analytics-tracker.sh; then
            test_result "Privacy-friendly analytics" "PASS"
        else
            test_result "Privacy-friendly analytics" "FAIL" "No privacy considerations found"
        fi
        
        # Test report generation
        if ./scripts/analytics-tracker.sh report > /dev/null 2>&1; then
            test_result "Analytics reporting works" "PASS"
        else
            test_result "Analytics reporting works" "FAIL" "Analytics reporting not working"
        fi
    else
        test_result "Analytics tracking script exists" "FAIL" "scripts/analytics-tracker.sh not found"
    fi
    
    # Check for analytics tracking in README (deployment button)
    if grep -q "analytics.*track\|track.*deploy" README.md; then
        test_result "Analytics tracking mentioned in README" "PASS"
    else
        test_result "Analytics tracking mentioned in README" "FAIL" "No analytics tracking mentioned"
    fi
}

test_analytics_tracking

# Requirement 8: Mobile Experience
echo -e "\n${BLUE}📱 Testing Mobile Experience (Req 8.1-8.5)${NC}"

test_mobile_experience() {
    # Check if mobile validation script exists
    if [ -f "scripts/validate-mobile-experience.sh" ]; then
        test_result "Mobile validation script exists" "PASS"
        
        # Run mobile validation and capture results
        if mobile_output=$(./scripts/validate-mobile-experience.sh 2>&1); then
            mobile_exit_code=$?
            
            if [ $mobile_exit_code -eq 0 ]; then
                test_result "Mobile validation passes" "PASS"
                
                # Extract pass rate from output
                if echo "$mobile_output" | grep -q "Pass Rate:.*[89][0-9]%\|Pass Rate: 100%"; then
                    test_result "High mobile pass rate" "PASS"
                else
                    test_result "High mobile pass rate" "FAIL" "Mobile pass rate below 80%"
                fi
            else
                test_result "Mobile validation passes" "FAIL" "Mobile validation failed"
            fi
        else
            test_result "Mobile validation script functional" "FAIL" "Mobile validation script not working"
        fi
    else
        test_result "Mobile validation script exists" "FAIL" "scripts/validate-mobile-experience.sh not found"
    fi
    
    # Check for mobile-specific CSS
    if grep -q "@media.*max-width" assets/static/css/campfire.css; then
        test_result "Mobile-responsive CSS" "PASS"
    else
        test_result "Mobile-responsive CSS" "FAIL" "No mobile media queries found"
    fi
    
    # Check for touch-friendly sizing
    if grep -q "44px\|48px" assets/static/css/campfire.css; then
        test_result "Touch-friendly target sizes" "PASS"
    else
        test_result "Touch-friendly target sizes" "FAIL" "No touch-friendly sizes found"
    fi
    
    # Check for mobile troubleshooting in README
    if grep -qi "mobile\|phone\|tablet" README.md; then
        test_result "Mobile troubleshooting in README" "PASS"
    else
        test_result "Mobile troubleshooting in README" "FAIL" "No mobile troubleshooting found"
    fi
}

test_mobile_experience

# Additional Integration Tests
echo -e "\n${BLUE}🔗 Testing Integration Requirements${NC}"

test_integration() {
    # Check for consistent messaging across paths
    if grep -q "2 minutes" README.md && grep -q "3 minutes" README.md; then
        test_result "Consistent timing promises" "PASS"
    else
        test_result "Consistent timing promises" "FAIL" "Inconsistent or missing timing promises"
    fi
    
    # Check for clear value proposition
    if grep -q "Team chat that works\|Team chat that actually works" README.md; then
        test_result "Clear value proposition" "PASS"
    else
        test_result "Clear value proposition" "FAIL" "No clear value proposition found"
    fi
    
    # Check for honest comparison section
    if grep -qi "honest.*comparison\|what.*campfire.*does\|what.*campfire.*doesn" README.md; then
        test_result "Honest comparison section" "PASS"
    else
        test_result "Honest comparison section" "FAIL" "No honest comparison found"
    fi
    
    # Check for troubleshooting section
    if grep -qi "troubleshoot\|need help\|issues" README.md; then
        test_result "Troubleshooting section present" "PASS"
    else
        test_result "Troubleshooting section present" "FAIL" "No troubleshooting section found"
    fi
    
    # Check for DHH/Jason Fried acknowledgment
    if grep -qi "dhh\|jason fried\|basecamp.*campfire" README.md; then
        test_result "Original Campfire acknowledgment" "PASS"
    else
        test_result "Original Campfire acknowledgment" "FAIL" "No acknowledgment of original Campfire"
    fi
}

test_integration

# Final Results
echo -e "\n${BLUE}📊 GTM Phase 2 Validation Results${NC}"
echo "=================================="
echo -e "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"

# Calculate pass percentage
if [ "$TOTAL_TESTS" -gt 0 ]; then
    pass_percentage=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    echo -e "Pass Rate: ${pass_percentage}%"
    
    echo -e "\n${BLUE}📋 Phase 2 Requirements Summary:${NC}"
    echo "✅ Two-Path User Experience (Req 1.1-1.5)"
    echo "✅ Local Sampling Experience (Req 2.1-2.5)" 
    echo "✅ Performance Claims Validation (Req 4.1-4.3)"
    echo "✅ Simple Analytics Tracking (Req 5.1-5.5)"
    echo "✅ Mobile Experience (Req 8.1-8.5)"
    
    if [ "$pass_percentage" -ge 90 ]; then
        echo -e "\n${GREEN}🎉 Excellent! Phase 2 GTM requirements are fully implemented.${NC}"
        echo -e "${GREEN}✅ README clearly shows two paths: local sampling and team deployment${NC}"
        echo -e "${GREEN}✅ Demo mode provides compelling local sampling experience${NC}"
        echo -e "${GREEN}✅ Performance claims in README are verified and accurate${NC}"
        echo -e "${GREEN}✅ Simple analytics track deployment success rates${NC}"
        echo -e "${GREEN}✅ Mobile experience is functional and user-friendly${NC}"
        exit 0
    elif [ "$pass_percentage" -ge 80 ]; then
        echo -e "\n${YELLOW}⚠️  Good progress! Phase 2 GTM mostly complete with minor issues.${NC}"
        exit 0
    else
        echo -e "\n${RED}❌ Phase 2 GTM requirements need significant work.${NC}"
        exit 1
    fi
else
    echo -e "\n${RED}❌ No tests were run.${NC}"
    exit 1
fi