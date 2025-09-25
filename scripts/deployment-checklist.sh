#!/bin/bash

# Campfire Deployment Checklist and Validation Script
# Comprehensive pre-deployment and post-deployment validation

set -euo pipefail

# Configuration
DEPLOYMENT_ENV="${CAMPFIRE_DEPLOYMENT_ENV:-production}"
CHECKLIST_VERSION="1.0.0"
VALIDATION_TIMEOUT=300

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] SUCCESS:${NC} $1"
}

# Checklist tracking
declare -A CHECKLIST_RESULTS
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# Record check result
record_check() {
    local check_name="$1"
    local result="$2"
    local details="${3:-}"
    
    CHECKLIST_RESULTS["$check_name"]="$result:$details"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if [[ "$result" == "PASS" ]]; then
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        success "✓ $check_name"
        [[ -n "$details" ]] && info "  $details"
    else
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        error "✗ $check_name"
        [[ -n "$details" ]] && error "  $details"
    fi
}

# Usage function
usage() {
    echo "Usage: $0 [PHASE] [OPTIONS]"
    echo ""
    echo "Deployment checklist and validation for Campfire"
    echo ""
    echo "Phases:"
    echo "  pre-deployment    Run pre-deployment checks"
    echo "  post-deployment   Run post-deployment validation"
    echo "  full             Run complete checklist"
    echo "  security         Run security-focused checks"
    echo "  performance      Run performance validation"
    echo ""
    echo "Options:"
    echo "  --env ENV        Deployment environment (default: production)"
    echo "  --timeout SEC    Validation timeout (default: 300)"
    echo "  --report-only    Generate report without running checks"
    echo "  --fix-issues     Attempt to fix detected issues"
    echo "  -h, --help       Show this help message"
    exit 1
}

# Parse command line arguments
PHASE="${1:-full}"
FIX_ISSUES=false
REPORT_ONLY=false

shift || true

while [[ $# -gt 0 ]]; do
    case $1 in
        --env)
            DEPLOYMENT_ENV="$2"
            shift 2
            ;;
        --timeout)
            VALIDATION_TIMEOUT="$2"
            shift 2
            ;;
        --fix-issues)
            FIX_ISSUES=true
            shift
            ;;
        --report-only)
            REPORT_ONLY=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            error "Unknown option: $1"
            usage
            ;;
    esac
done

# Pre-deployment checks
run_pre_deployment_checks() {
    log "Running pre-deployment checks..."
    
    # 1. Environment Configuration
    check_environment_configuration
    
    # 2. Dependencies
    check_dependencies
    
    # 3. Resources
    check_system_resources
    
    # 4. Security
    check_security_configuration
    
    # 5. Backup
    check_backup_readiness
    
    # 6. Network
    check_network_configuration
    
    # 7. SSL/TLS
    check_ssl_configuration
    
    # 8. Database
    check_database_readiness
}

# Post-deployment validation
run_post_deployment_validation() {
    log "Running post-deployment validation..."
    
    # 1. Application Health
    check_application_health
    
    # 2. Service Availability
    check_service_availability
    
    # 3. Performance
    check_performance_metrics
    
    # 4. Security
    check_security_posture
    
    # 5. Monitoring
    check_monitoring_setup
    
    # 6. Backup Functionality
    check_backup_functionality
    
    # 7. Integration Tests
    run_integration_tests
    
    # 8. Load Testing
    run_basic_load_test
}

# Environment configuration checks
check_environment_configuration() {
    info "Checking environment configuration..."
    
    # Check environment file exists
    local env_file=".env.${DEPLOYMENT_ENV}"
    if [[ -f "$env_file" ]]; then
        record_check "Environment file exists" "PASS" "$env_file found"
    else
        record_check "Environment file exists" "FAIL" "$env_file not found"
        return
    fi
    
    # Check required environment variables
    local required_vars=(
        "CAMPFIRE_HOST"
        "CAMPFIRE_PORT"
        "CAMPFIRE_DATABASE_URL"
        "CAMPFIRE_LOG_LEVEL"
    )
    
    local missing_vars=()
    for var in "${required_vars[@]}"; do
        if ! grep -q "^${var}=" "$env_file"; then
            missing_vars+=("$var")
        fi
    done
    
    if [[ ${#missing_vars[@]} -eq 0 ]]; then
        record_check "Required environment variables" "PASS" "All required variables present"
    else
        record_check "Required environment variables" "FAIL" "Missing: ${missing_vars[*]}"
    fi
    
    # Check VAPID keys for push notifications
    if grep -q "CAMPFIRE_PUSH_ENABLED=true" "$env_file"; then
        if grep -q "CAMPFIRE_VAPID_PRIVATE_KEY=" "$env_file" && grep -q "CAMPFIRE_VAPID_PUBLIC_KEY=" "$env_file"; then
            record_check "VAPID keys configured" "PASS" "Push notification keys present"
        else
            record_check "VAPID keys configured" "FAIL" "Push notifications enabled but VAPID keys missing"
        fi
    else
        record_check "VAPID keys configured" "PASS" "Push notifications disabled"
    fi
}

# Dependencies check
check_dependencies() {
    info "Checking dependencies..."
    
    local required_commands=("docker" "docker-compose" "curl" "jq")
    local missing_commands=()
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_commands+=("$cmd")
        fi
    done
    
    if [[ ${#missing_commands[@]} -eq 0 ]]; then
        record_check "Required commands available" "PASS" "All commands found"
    else
        record_check "Required commands available" "FAIL" "Missing: ${missing_commands[*]}"
        
        if [[ "$FIX_ISSUES" == "true" ]]; then
            info "Attempting to install missing dependencies..."
            # Add installation logic here
        fi
    fi
    
    # Check Docker daemon
    if docker info &> /dev/null; then
        record_check "Docker daemon running" "PASS" "Docker is accessible"
    else
        record_check "Docker daemon running" "FAIL" "Docker daemon not running or not accessible"
    fi
    
    # Check Docker Compose version
    local compose_version=$(docker-compose version --short 2>/dev/null || echo "unknown")
    if [[ "$compose_version" != "unknown" ]]; then
        record_check "Docker Compose version" "PASS" "Version: $compose_version"
    else
        record_check "Docker Compose version" "FAIL" "Could not determine version"
    fi
}

# System resources check
check_system_resources() {
    info "Checking system resources..."
    
    # Check available memory
    local available_memory_mb=$(free -m | awk 'NR==2{print $7}')
    local required_memory_mb=1024
    
    if [[ $available_memory_mb -ge $required_memory_mb ]]; then
        record_check "Available memory" "PASS" "${available_memory_mb}MB available (required: ${required_memory_mb}MB)"
    else
        record_check "Available memory" "FAIL" "Only ${available_memory_mb}MB available (required: ${required_memory_mb}MB)"
    fi
    
    # Check available disk space
    local available_disk_gb=$(df -BG . | awk 'NR==2{print $4}' | sed 's/G//')
    local required_disk_gb=10
    
    if [[ $available_disk_gb -ge $required_disk_gb ]]; then
        record_check "Available disk space" "PASS" "${available_disk_gb}GB available (required: ${required_disk_gb}GB)"
    else
        record_check "Available disk space" "FAIL" "Only ${available_disk_gb}GB available (required: ${required_disk_gb}GB)"
    fi
    
    # Check CPU cores
    local cpu_cores=$(nproc)
    local required_cores=2
    
    if [[ $cpu_cores -ge $required_cores ]]; then
        record_check "CPU cores" "PASS" "${cpu_cores} cores available (required: ${required_cores})"
    else
        record_check "CPU cores" "FAIL" "Only ${cpu_cores} cores available (required: ${required_cores})"
    fi
}

# Security configuration check
check_security_configuration() {
    info "Checking security configuration..."
    
    # Check file permissions
    local sensitive_files=(".env.${DEPLOYMENT_ENV}" "docker-compose.yml")
    local permission_issues=()
    
    for file in "${sensitive_files[@]}"; do
        if [[ -f "$file" ]]; then
            local perms=$(stat -c "%a" "$file")
            if [[ "$perms" -le 600 ]]; then
                record_check "File permissions: $file" "PASS" "Permissions: $perms"
            else
                permission_issues+=("$file:$perms")
            fi
        fi
    done
    
    if [[ ${#permission_issues[@]} -eq 0 ]]; then
        record_check "File permissions" "PASS" "All files have secure permissions"
    else
        record_check "File permissions" "FAIL" "Issues: ${permission_issues[*]}"
        
        if [[ "$FIX_ISSUES" == "true" ]]; then
            info "Fixing file permissions..."
            chmod 600 ".env.${DEPLOYMENT_ENV}" 2>/dev/null || true
        fi
    fi
    
    # Check for secrets in environment file
    local env_file=".env.${DEPLOYMENT_ENV}"
    if [[ -f "$env_file" ]]; then
        local weak_secrets=()
        
        # Check for default/weak passwords
        if grep -q "password.*=.*admin\|password.*=.*123" "$env_file"; then
            weak_secrets+=("weak_password")
        fi
        
        # Check for empty secrets
        if grep -q ".*_KEY=\s*$\|.*_SECRET=\s*$" "$env_file"; then
            weak_secrets+=("empty_secrets")
        fi
        
        if [[ ${#weak_secrets[@]} -eq 0 ]]; then
            record_check "Secret strength" "PASS" "No weak secrets detected"
        else
            record_check "Secret strength" "FAIL" "Issues: ${weak_secrets[*]}"
        fi
    fi
}

# Backup readiness check
check_backup_readiness() {
    info "Checking backup readiness..."
    
    # Check backup directory exists
    local backup_dir="./backups"
    if [[ -d "$backup_dir" ]]; then
        record_check "Backup directory exists" "PASS" "$backup_dir found"
    else
        record_check "Backup directory exists" "FAIL" "$backup_dir not found"
        
        if [[ "$FIX_ISSUES" == "true" ]]; then
            mkdir -p "$backup_dir"
            info "Created backup directory: $backup_dir"
        fi
    fi
    
    # Check backup script exists
    if [[ -f "scripts/backup-enhanced.sh" ]]; then
        record_check "Backup script available" "PASS" "Enhanced backup script found"
    else
        record_check "Backup script available" "FAIL" "Backup script not found"
    fi
    
    # Check backup script permissions
    if [[ -x "scripts/backup-enhanced.sh" ]]; then
        record_check "Backup script executable" "PASS" "Script has execute permissions"
    else
        record_check "Backup script executable" "FAIL" "Script not executable"
        
        if [[ "$FIX_ISSUES" == "true" ]]; then
            chmod +x "scripts/backup-enhanced.sh"
            info "Made backup script executable"
        fi
    fi
}

# Network configuration check
check_network_configuration() {
    info "Checking network configuration..."
    
    # Check port availability
    local required_ports=(3000 9090 3001)
    local port_conflicts=()
    
    for port in "${required_ports[@]}"; do
        if netstat -tuln 2>/dev/null | grep -q ":${port} "; then
            port_conflicts+=("$port")
        fi
    done
    
    if [[ ${#port_conflicts[@]} -eq 0 ]]; then
        record_check "Port availability" "PASS" "All required ports available"
    else
        record_check "Port availability" "FAIL" "Ports in use: ${port_conflicts[*]}"
    fi
    
    # Check DNS resolution
    if nslookup google.com &> /dev/null; then
        record_check "DNS resolution" "PASS" "DNS is working"
    else
        record_check "DNS resolution" "FAIL" "DNS resolution failed"
    fi
    
    # Check internet connectivity
    if curl -s --max-time 10 https://httpbin.org/ip &> /dev/null; then
        record_check "Internet connectivity" "PASS" "External connectivity working"
    else
        record_check "Internet connectivity" "FAIL" "Cannot reach external services"
    fi
}

# SSL configuration check
check_ssl_configuration() {
    info "Checking SSL configuration..."
    
    # Check if SSL is configured
    local env_file=".env.${DEPLOYMENT_ENV}"
    if grep -q "CAMPFIRE_FORCE_HTTPS=true" "$env_file" 2>/dev/null; then
        # Check for SSL certificates or Traefik configuration
        if [[ -f "docker-compose.yml" ]] && grep -q "traefik" "docker-compose.yml"; then
            record_check "SSL configuration" "PASS" "Traefik SSL configured"
        elif [[ -d "certs" ]] && [[ -f "certs/cert.pem" ]] && [[ -f "certs/key.pem" ]]; then
            record_check "SSL configuration" "PASS" "Manual SSL certificates found"
        else
            record_check "SSL configuration" "FAIL" "HTTPS enabled but no SSL configuration found"
        fi
    else
        record_check "SSL configuration" "PASS" "HTTPS not enforced (development mode)"
    fi
}

# Database readiness check
check_database_readiness() {
    info "Checking database readiness..."
    
    # Check if database directory exists
    local db_dir="./data"
    if [[ -d "$db_dir" ]]; then
        record_check "Database directory exists" "PASS" "$db_dir found"
    else
        record_check "Database directory exists" "FAIL" "$db_dir not found"
        
        if [[ "$FIX_ISSUES" == "true" ]]; then
            mkdir -p "$db_dir"
            info "Created database directory: $db_dir"
        fi
    fi
    
    # Check database file if it exists
    local db_file="./data/campfire.db"
    if [[ -f "$db_file" ]]; then
        # Check database integrity
        if sqlite3 "$db_file" "PRAGMA integrity_check;" | grep -q "ok"; then
            record_check "Database integrity" "PASS" "Database integrity check passed"
        else
            record_check "Database integrity" "FAIL" "Database integrity check failed"
        fi
        
        # Check database size
        local db_size=$(du -h "$db_file" | cut -f1)
        record_check "Database size" "PASS" "Database size: $db_size"
    else
        record_check "Database file" "PASS" "No existing database (fresh installation)"
    fi
}

# Application health check
check_application_health() {
    info "Checking application health..."
    
    # Wait for application to start
    local max_attempts=30
    local attempt=0
    
    while [[ $attempt -lt $max_attempts ]]; do
        if curl -f -s http://localhost:3000/health &> /dev/null; then
            break
        fi
        sleep 2
        attempt=$((attempt + 1))
    done
    
    if [[ $attempt -lt $max_attempts ]]; then
        record_check "Application startup" "PASS" "Application started successfully"
    else
        record_check "Application startup" "FAIL" "Application failed to start within ${max_attempts} attempts"
        return
    fi
    
    # Check health endpoint
    local health_response=$(curl -s http://localhost:3000/health)
    if echo "$health_response" | jq -e '.status == "healthy"' &> /dev/null; then
        record_check "Health endpoint" "PASS" "Health check passed"
    else
        record_check "Health endpoint" "FAIL" "Health check failed: $health_response"
    fi
    
    # Check detailed health
    local detailed_health=$(curl -s http://localhost:3000/health/detailed)
    if echo "$detailed_health" | jq -e '.status == "healthy"' &> /dev/null; then
        record_check "Detailed health check" "PASS" "All components healthy"
    else
        record_check "Detailed health check" "FAIL" "Some components unhealthy"
    fi
}

# Service availability check
check_service_availability() {
    info "Checking service availability..."
    
    # Check main application
    if curl -f -s http://localhost:3000/health &> /dev/null; then
        record_check "Main application" "PASS" "Application responding"
    else
        record_check "Main application" "FAIL" "Application not responding"
    fi
    
    # Check metrics endpoint
    if curl -f -s http://localhost:3000/metrics &> /dev/null; then
        record_check "Metrics endpoint" "PASS" "Metrics available"
    else
        record_check "Metrics endpoint" "FAIL" "Metrics not available"
    fi
    
    # Check WebSocket endpoint (if wscat is available)
    if command -v wscat &> /dev/null; then
        if timeout 10 wscat -c ws://localhost:3000/ws -x '{"type":"ping"}' &> /dev/null; then
            record_check "WebSocket endpoint" "PASS" "WebSocket connection successful"
        else
            record_check "WebSocket endpoint" "FAIL" "WebSocket connection failed"
        fi
    else
        record_check "WebSocket endpoint" "PASS" "wscat not available, skipping test"
    fi
    
    # Check monitoring services (if running)
    if docker ps -q -f name=prometheus | grep -q .; then
        if curl -f -s http://localhost:9090/-/ready &> /dev/null; then
            record_check "Prometheus monitoring" "PASS" "Prometheus ready"
        else
            record_check "Prometheus monitoring" "FAIL" "Prometheus not ready"
        fi
    else
        record_check "Prometheus monitoring" "PASS" "Prometheus not configured"
    fi
    
    if docker ps -q -f name=grafana | grep -q .; then
        if curl -f -s http://localhost:3001/api/health &> /dev/null; then
            record_check "Grafana dashboard" "PASS" "Grafana ready"
        else
            record_check "Grafana dashboard" "FAIL" "Grafana not ready"
        fi
    else
        record_check "Grafana dashboard" "PASS" "Grafana not configured"
    fi
}

# Performance metrics check
check_performance_metrics() {
    info "Checking performance metrics..."
    
    # Check response time
    local response_time=$(curl -o /dev/null -s -w '%{time_total}' http://localhost:3000/health)
    local response_time_ms=$(echo "$response_time * 1000" | bc -l | cut -d'.' -f1)
    
    if [[ $response_time_ms -lt 1000 ]]; then
        record_check "Response time" "PASS" "${response_time_ms}ms (target: <1000ms)"
    else
        record_check "Response time" "FAIL" "${response_time_ms}ms (target: <1000ms)"
    fi
    
    # Check memory usage
    if docker ps -q -f name=campfire | grep -q .; then
        local memory_usage=$(docker stats --no-stream --format "{{.MemUsage}}" campfire | cut -d'/' -f1)
        record_check "Memory usage" "PASS" "Current usage: $memory_usage"
    else
        record_check "Memory usage" "FAIL" "Cannot get memory usage - container not running"
    fi
    
    # Check CPU usage
    if docker ps -q -f name=campfire | grep -q .; then
        local cpu_usage=$(docker stats --no-stream --format "{{.CPUPerc}}" campfire)
        record_check "CPU usage" "PASS" "Current usage: $cpu_usage"
    else
        record_check "CPU usage" "FAIL" "Cannot get CPU usage - container not running"
    fi
}

# Security posture check
check_security_posture() {
    info "Checking security posture..."
    
    # Check if running as non-root
    if docker exec campfire whoami 2>/dev/null | grep -q campfire; then
        record_check "Non-root user" "PASS" "Application running as non-root user"
    else
        record_check "Non-root user" "FAIL" "Application may be running as root"
    fi
    
    # Check security headers
    local security_headers=$(curl -I -s http://localhost:3000/health)
    
    if echo "$security_headers" | grep -q "X-Content-Type-Options"; then
        record_check "Security headers" "PASS" "Security headers present"
    else
        record_check "Security headers" "FAIL" "Missing security headers"
    fi
    
    # Check for exposed sensitive endpoints
    local sensitive_endpoints=("/admin" "/debug" "/.env")
    local exposed_endpoints=()
    
    for endpoint in "${sensitive_endpoints[@]}"; do
        if curl -f -s "http://localhost:3000${endpoint}" &> /dev/null; then
            exposed_endpoints+=("$endpoint")
        fi
    done
    
    if [[ ${#exposed_endpoints[@]} -eq 0 ]]; then
        record_check "Sensitive endpoints" "PASS" "No sensitive endpoints exposed"
    else
        record_check "Sensitive endpoints" "FAIL" "Exposed: ${exposed_endpoints[*]}"
    fi
}

# Monitoring setup check
check_monitoring_setup() {
    info "Checking monitoring setup..."
    
    # Check if monitoring is configured
    if [[ -f "monitoring/prometheus.yml" ]]; then
        record_check "Prometheus configuration" "PASS" "Configuration file found"
    else
        record_check "Prometheus configuration" "FAIL" "Configuration file not found"
    fi
    
    # Check alerting rules
    if [[ -f "monitoring/rules/campfire.yml" ]]; then
        record_check "Alerting rules" "PASS" "Alerting rules configured"
    else
        record_check "Alerting rules" "FAIL" "No alerting rules found"
    fi
    
    # Check Grafana dashboards
    if [[ -d "monitoring/grafana/dashboards" ]] && [[ -n "$(ls -A monitoring/grafana/dashboards 2>/dev/null)" ]]; then
        record_check "Grafana dashboards" "PASS" "Dashboards configured"
    else
        record_check "Grafana dashboards" "FAIL" "No dashboards found"
    fi
}

# Backup functionality check
check_backup_functionality() {
    info "Checking backup functionality..."
    
    # Test backup script
    if [[ -x "scripts/backup-enhanced.sh" ]]; then
        if ./scripts/backup-enhanced.sh --dry-run &> /dev/null; then
            record_check "Backup script test" "PASS" "Backup script dry run successful"
        else
            record_check "Backup script test" "FAIL" "Backup script dry run failed"
        fi
    else
        record_check "Backup script test" "FAIL" "Backup script not executable"
    fi
    
    # Check backup directory permissions
    if [[ -d "./backups" ]] && [[ -w "./backups" ]]; then
        record_check "Backup directory writable" "PASS" "Backup directory is writable"
    else
        record_check "Backup directory writable" "FAIL" "Backup directory not writable"
    fi
}

# Integration tests
run_integration_tests() {
    info "Running integration tests..."
    
    # Test API endpoints
    local api_tests=(
        "GET:/health:200"
        "GET:/health/ready:200"
        "GET:/health/detailed:200"
        "GET:/metrics:200"
    )
    
    local failed_tests=()
    
    for test in "${api_tests[@]}"; do
        local method=$(echo "$test" | cut -d':' -f1)
        local endpoint=$(echo "$test" | cut -d':' -f2)
        local expected_status=$(echo "$test" | cut -d':' -f3)
        
        local actual_status=$(curl -s -o /dev/null -w '%{http_code}' -X "$method" "http://localhost:3000${endpoint}")
        
        if [[ "$actual_status" == "$expected_status" ]]; then
            info "✓ $method $endpoint -> $actual_status"
        else
            failed_tests+=("$method $endpoint: expected $expected_status, got $actual_status")
        fi
    done
    
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        record_check "API integration tests" "PASS" "All API tests passed"
    else
        record_check "API integration tests" "FAIL" "Failed tests: ${#failed_tests[@]}"
    fi
}

# Basic load test
run_basic_load_test() {
    info "Running basic load test..."
    
    if command -v ab &> /dev/null; then
        # Run Apache Bench test
        local ab_result=$(ab -n 100 -c 10 http://localhost:3000/health 2>&1)
        
        if echo "$ab_result" | grep -q "Complete requests.*100"; then
            local avg_time=$(echo "$ab_result" | grep "Time per request" | head -1 | awk '{print $4}')
            record_check "Basic load test" "PASS" "100 requests completed, avg: ${avg_time}ms"
        else
            record_check "Basic load test" "FAIL" "Load test failed"
        fi
    else
        record_check "Basic load test" "PASS" "Apache Bench not available, skipping"
    fi
}

# Generate comprehensive report
generate_report() {
    local report_file="deployment-checklist-report-$(date +%Y%m%d_%H%M%S).html"
    
    cat > "$report_file" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Campfire Deployment Checklist Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .summary { margin: 20px 0; }
        .pass { color: green; }
        .fail { color: red; }
        .check-item { margin: 10px 0; padding: 10px; border-left: 4px solid #ddd; }
        .check-item.pass { border-left-color: green; background-color: #f0fff0; }
        .check-item.fail { border-left-color: red; background-color: #fff0f0; }
        .details { font-size: 0.9em; color: #666; margin-top: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Campfire Deployment Checklist Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Environment:</strong> $DEPLOYMENT_ENV</p>
        <p><strong>Checklist Version:</strong> $CHECKLIST_VERSION</p>
    </div>
    
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Total Checks:</strong> $TOTAL_CHECKS</p>
        <p><strong>Passed:</strong> <span class="pass">$PASSED_CHECKS</span></p>
        <p><strong>Failed:</strong> <span class="fail">$FAILED_CHECKS</span></p>
        <p><strong>Success Rate:</strong> $(( PASSED_CHECKS * 100 / TOTAL_CHECKS ))%</p>
    </div>
    
    <div class="results">
        <h2>Detailed Results</h2>
EOF

    # Add check results
    for check_name in "${!CHECKLIST_RESULTS[@]}"; do
        local result_data="${CHECKLIST_RESULTS[$check_name]}"
        local result=$(echo "$result_data" | cut -d':' -f1)
        local details=$(echo "$result_data" | cut -d':' -f2-)
        
        local css_class=$(echo "$result" | tr '[:upper:]' '[:lower:]')
        
        cat >> "$report_file" <<EOF
        <div class="check-item $css_class">
            <strong>$check_name:</strong> <span class="$css_class">$result</span>
            $(if [[ -n "$details" ]]; then echo "<div class=\"details\">$details</div>"; fi)
        </div>
EOF
    done
    
    cat >> "$report_file" <<EOF
    </div>
</body>
</html>
EOF

    log "Report generated: $report_file"
}

# Main execution
main() {
    log "Starting Campfire deployment checklist..."
    info "Phase: $PHASE"
    info "Environment: $DEPLOYMENT_ENV"
    info "Fix issues: $FIX_ISSUES"
    
    if [[ "$REPORT_ONLY" == "true" ]]; then
        generate_report
        exit 0
    fi
    
    case "$PHASE" in
        "pre-deployment")
            run_pre_deployment_checks
            ;;
        "post-deployment")
            run_post_deployment_validation
            ;;
        "full")
            run_pre_deployment_checks
            run_post_deployment_validation
            ;;
        "security")
            check_security_configuration
            check_security_posture
            ;;
        "performance")
            check_performance_metrics
            run_basic_load_test
            ;;
        *)
            error "Unknown phase: $PHASE"
            usage
            ;;
    esac
    
    # Generate report
    generate_report
    
    # Summary
    echo ""
    log "Deployment checklist completed"
    info "Total checks: $TOTAL_CHECKS"
    success "Passed: $PASSED_CHECKS"
    if [[ $FAILED_CHECKS -gt 0 ]]; then
        error "Failed: $FAILED_CHECKS"
        exit 1
    else
        success "All checks passed!"
    fi
}

# Run main function
main "$@"