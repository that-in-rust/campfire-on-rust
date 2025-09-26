#!/bin/bash

# Performance Validation Script for Campfire
# Validates all performance claims made in README.md

set -e

echo "🔥 Campfire Performance Validation"
echo "=================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Results tracking
RESULTS_FILE="performance_results.json"
echo "{" > $RESULTS_FILE

# Function to log results
log_result() {
    local test_name="$1"
    local value="$2"
    local unit="$3"
    local status="$4"
    
    echo "  \"$test_name\": {" >> $RESULTS_FILE
    echo "    \"value\": $value," >> $RESULTS_FILE
    echo "    \"unit\": \"$unit\"," >> $RESULTS_FILE
    echo "    \"status\": \"$status\"," >> $RESULTS_FILE
    echo "    \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"" >> $RESULTS_FILE
    echo "  }," >> $RESULTS_FILE
}

# 1. Test Startup Time (Claim: < 1 second)
echo -e "${BLUE}1. Testing Startup Time${NC}"
echo "   Claim: Starts in under 1 second"

# Build release version first
echo "   Building release version..."
cargo build --release --quiet

# Test startup time
echo "   Measuring startup time..."
START_TIME=$(date +%s.%N)

# Start the application in background
cargo run --release -- --port 3001 > /dev/null 2>&1 &
APP_PID=$!

# Wait for the application to be ready (check if port is listening)
timeout=10
counter=0
while ! nc -z localhost 3001 2>/dev/null; do
    sleep 0.1
    counter=$((counter + 1))
    if [ $counter -gt $((timeout * 10)) ]; then
        echo -e "   ${RED}✗ Application failed to start within ${timeout}s${NC}"
        kill $APP_PID 2>/dev/null || true
        exit 1
    fi
done

END_TIME=$(date +%s.%N)
STARTUP_TIME=$(echo "$END_TIME - $START_TIME" | bc)

# Clean up
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true

if (( $(echo "$STARTUP_TIME < 1.0" | bc -l) )); then
    echo -e "   ${GREEN}✓ Startup time: ${STARTUP_TIME}s (< 1s)${NC}"
    log_result "startup_time" "$STARTUP_TIME" "seconds" "PASS"
else
    echo -e "   ${RED}✗ Startup time: ${STARTUP_TIME}s (>= 1s)${NC}"
    log_result "startup_time" "$STARTUP_TIME" "seconds" "FAIL"
fi

echo ""

# 2. Test Memory Usage (Claim: ~20MB base)
echo -e "${BLUE}2. Testing Memory Usage${NC}"
echo "   Claim: Uses ~20MB RAM base"

# Start application and measure memory
cargo run --release -- --port 3002 > /dev/null 2>&1 &
APP_PID=$!

# Wait for startup
sleep 2

# Get memory usage (RSS in KB)
if ps -p $APP_PID > /dev/null 2>&1; then
    MEMORY_KB=$(ps -o rss= -p $APP_PID | tr -d ' ')
    MEMORY_MB=$(echo "scale=2; $MEMORY_KB / 1024" | bc)
    
    if (( $(echo "$MEMORY_MB <= 30" | bc -l) )); then
        echo -e "   ${GREEN}✓ Memory usage: ${MEMORY_MB}MB (≤ 30MB acceptable)${NC}"
        log_result "base_memory" "$MEMORY_MB" "MB" "PASS"
    else
        echo -e "   ${YELLOW}⚠ Memory usage: ${MEMORY_MB}MB (> 30MB, higher than claimed)${NC}"
        log_result "base_memory" "$MEMORY_MB" "MB" "WARNING"
    fi
else
    echo -e "   ${RED}✗ Could not measure memory usage${NC}"
    log_result "base_memory" "0" "MB" "FAIL"
fi

# Clean up
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true

echo ""

# 3. Test Binary Size
echo -e "${BLUE}3. Testing Binary Size${NC}"

BINARY_SIZE=$(stat -f%z target/release/campfire 2>/dev/null || stat -c%s target/release/campfire 2>/dev/null || echo "0")
BINARY_SIZE_MB=$(echo "scale=2; $BINARY_SIZE / 1024 / 1024" | bc)

echo "   Binary size: ${BINARY_SIZE_MB}MB"
log_result "binary_size" "$BINARY_SIZE_MB" "MB" "INFO"

echo ""

# 4. Test Database Performance (Search claim: <10ms for 10,000+ messages)
echo -e "${BLUE}4. Testing Database Performance${NC}"
echo "   Note: This requires a populated database for accurate testing"

# Start application for database tests
cargo run --release -- --port 3003 > /dev/null 2>&1 &
APP_PID=$!
sleep 2

# Simple HTTP response time test
if command -v curl > /dev/null 2>&1; then
    echo "   Testing HTTP response time..."
    RESPONSE_TIME=$(curl -o /dev/null -s -w '%{time_total}' http://localhost:3003/ || echo "0")
    RESPONSE_TIME_MS=$(echo "$RESPONSE_TIME * 1000" | bc)
    
    if (( $(echo "$RESPONSE_TIME_MS < 100" | bc -l) )); then
        echo -e "   ${GREEN}✓ HTTP response time: ${RESPONSE_TIME_MS}ms (< 100ms)${NC}"
        log_result "http_response_time" "$RESPONSE_TIME_MS" "ms" "PASS"
    else
        echo -e "   ${YELLOW}⚠ HTTP response time: ${RESPONSE_TIME_MS}ms (>= 100ms)${NC}"
        log_result "http_response_time" "$RESPONSE_TIME_MS" "ms" "WARNING"
    fi
else
    echo "   curl not available, skipping HTTP response test"
fi

# Clean up
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true

echo ""

# 5. Compilation Time
echo -e "${BLUE}5. Testing Compilation Performance${NC}"

echo "   Testing clean build time..."
cargo clean --quiet

START_TIME=$(date +%s.%N)
cargo build --release --quiet
END_TIME=$(date +%s.%N)

BUILD_TIME=$(echo "$END_TIME - $START_TIME" | bc)
echo "   Clean build time: ${BUILD_TIME}s"
log_result "build_time" "$BUILD_TIME" "seconds" "INFO"

echo ""

# Close JSON file
sed -i '$ s/,$//' $RESULTS_FILE 2>/dev/null || sed -i '' '$ s/,$//' $RESULTS_FILE 2>/dev/null || true
echo "}" >> $RESULTS_FILE

# Summary
echo -e "${BLUE}Performance Validation Summary${NC}"
echo "============================="
echo ""
echo "Results saved to: $RESULTS_FILE"
echo ""
echo -e "${GREEN}✓ PASS${NC} - Meets performance claim"
echo -e "${YELLOW}⚠ WARNING${NC} - Exceeds claimed performance but acceptable"
echo -e "${RED}✗ FAIL${NC} - Does not meet performance claim"
echo ""

# Generate performance report
cat > performance_report.md << EOF
# Campfire Performance Validation Report

Generated: $(date)

## Test Results

### Startup Time
- **Claim**: Starts in under 1 second
- **Measured**: ${STARTUP_TIME}s
- **Status**: $(if (( $(echo "$STARTUP_TIME < 1.0" | bc -l) )); then echo "✅ PASS"; else echo "❌ FAIL"; fi)
- **Startup time verified**: $(if (( $(echo "$STARTUP_TIME < 1.0" | bc -l) )); then echo "YES - ${STARTUP_TIME}s < 1s"; else echo "NO - ${STARTUP_TIME}s >= 1s"; fi)

### Memory Usage
- **Claim**: Uses ~20MB RAM
- **Measured**: ${MEMORY_MB}MB
- **Status**: $(if (( $(echo "$MEMORY_MB <= 30" | bc -l) )); then echo "✅ PASS"; else echo "⚠️ WARNING"; fi)
- **Memory usage verified**: $(if (( $(echo "$MEMORY_MB <= 30" | bc -l) )); then echo "YES - ${MEMORY_MB}MB <= 30MB"; else echo "NO - ${MEMORY_MB}MB > 30MB"; fi)

### Binary Size
- **Measured**: ${BINARY_SIZE_MB}MB
- **Status**: ℹ️ INFO

### HTTP Response Time
- **Measured**: ${RESPONSE_TIME_MS}ms
- **Status**: $(if (( $(echo "$RESPONSE_TIME_MS < 100" | bc -l) )); then echo "✅ PASS"; else echo "⚠️ WARNING"; fi)

### Build Time
- **Measured**: ${BUILD_TIME}s
- **Status**: ℹ️ INFO

## Recommendations

Based on these results, the README should be updated with:
1. Verified startup time: ${STARTUP_TIME}s
2. Actual memory usage: ${MEMORY_MB}MB
3. Remove unsubstantiated claims about concurrent users and search performance until properly tested

## Raw Data

See \`performance_results.json\` for detailed measurements.
EOF

echo "Performance report generated: performance_report.md"
echo ""
echo -e "${BLUE}Next Steps:${NC}"
echo "1. Review performance_report.md"
echo "2. Update README.md with verified numbers only"
echo "3. Remove unsubstantiated claims"
echo "4. Add benchmarking tests to CI/CD pipeline"