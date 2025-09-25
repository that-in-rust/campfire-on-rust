# Backup and Restore Procedures for Campfire Rust

This document provides comprehensive backup and restore procedures for Campfire deployments, including automated backup strategies, disaster recovery, and data migration procedures.

## Table of Contents

1. [Backup Strategy Overview](#backup-strategy-overview)
2. [Automated Backup Setup](#automated-backup-setup)
3. [Manual Backup Procedures](#manual-backup-procedures)
4. [Restore Procedures](#restore-procedures)
5. [Disaster Recovery](#disaster-recovery)
6. [Data Migration](#data-migration)
7. [Backup Verification](#backup-verification)
8. [Remote Backup Storage](#remote-backup-storage)
9. [Monitoring and Alerting](#monitoring-and-alerting)
10. [Best Practices](#best-practices)

## Backup Strategy Overview

### Backup Types

Campfire supports multiple backup types to meet different recovery requirements:

| Backup Type | Description | Use Case | Frequency |
|-------------|-------------|----------|-----------|
| **Full** | Complete database backup | Disaster recovery, migrations | Daily |
| **Incremental** | Changes since last full backup | Point-in-time recovery | Hourly |
| **Hot** | Backup while application running | Zero-downtime backups | Continuous |
| **Schema** | Database structure only | Development, testing | Weekly |

### Retention Policy

Default retention policy (configurable):

- **Daily backups**: 30 days
- **Weekly backups**: 12 weeks  
- **Monthly backups**: 12 months
- **Yearly backups**: 7 years

### Recovery Point Objective (RPO) and Recovery Time Objective (RTO)

| Deployment Size | RPO | RTO | Backup Frequency |
|----------------|-----|-----|------------------|
| **Small** (< 100 users) | 24 hours | 1 hour | Daily |
| **Medium** (< 1,000 users) | 4 hours | 30 minutes | Every 4 hours |
| **Large** (< 10,000 users) | 1 hour | 15 minutes | Hourly |
| **Enterprise** (10,000+ users) | 15 minutes | 5 minutes | Continuous |

## Automated Backup Setup

### Production Backup Schedule

```bash
#!/bin/bash
# /opt/campfire/scripts/backup-schedule.sh

# Daily full backup at 2 AM
0 2 * * * /opt/campfire/scripts/backup-enhanced.sh full --compress --verify --remote

# Hourly incremental backups during business hours
0 9-17 * * 1-5 /opt/campfire/scripts/backup-enhanced.sh incremental --compress

# Weekly schema backup on Sundays
0 1 * * 0 /opt/campfire/scripts/backup-enhanced.sh schema

# Monthly cleanup of old backups
0 3 1 * * /opt/campfire/scripts/cleanup-old-backups.sh
```

### Docker-based Automated Backup

Create a backup service in `docker-compose.backup.yml`:

```yaml
version: '3.8'

services:
  backup:
    image: campfire-on-rust:latest
    container_name: campfire-backup
    restart: "no"
    profiles:
      - backup
    
    volumes:
      - campfire_data:/app/data:ro
      - campfire_backups:/app/backups
      - ./scripts:/app/scripts:ro
    
    environment:
      - CAMPFIRE_DATABASE_URL=/app/data/campfire.db
      - CAMPFIRE_BACKUP_DIR=/app/backups
      - CAMPFIRE_BACKUP_RETENTION_DAYS=30
      - CAMPFIRE_S3_BUCKET=${BACKUP_S3_BUCKET}
      - AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}
      - AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}
    
    command: /app/scripts/backup-enhanced.sh full --compress --verify --remote
    
    depends_on:
      - campfire

  # Backup scheduler using cron
  backup-scheduler:
    image: alpine:latest
    container_name: campfire-backup-scheduler
    restart: unless-stopped
    profiles:
      - backup
    
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./scripts:/scripts:ro
    
    environment:
      - COMPOSE_PROJECT_NAME=${COMPOSE_PROJECT_NAME:-campfire}
    
    command: >
      sh -c "
        apk add --no-cache docker-cli dcron &&
        echo '0 2 * * * docker-compose -f /scripts/docker-compose.backup.yml --profile backup run --rm backup' > /etc/crontabs/root &&
        echo '0 */4 * * * docker-compose -f /scripts/docker-compose.backup.yml --profile backup run --rm backup incremental' >> /etc/crontabs/root &&
        crond -f -l 2
      "
```

### Kubernetes CronJob for Backup

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: campfire-backup
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: campfire-on-rust:latest
            command: ["/app/scripts/backup-enhanced.sh"]
            args: ["full", "--compress", "--verify", "--remote"]
            env:
            - name: CAMPFIRE_DATABASE_URL
              value: "/app/data/campfire.db"
            - name: CAMPFIRE_S3_BUCKET
              valueFrom:
                secretKeyRef:
                  name: backup-secrets
                  key: s3-bucket
            volumeMounts:
            - name: data-volume
              mountPath: /app/data
              readOnly: true
            - name: backup-volume
              mountPath: /app/backups
          volumes:
          - name: data-volume
            persistentVolumeClaim:
              claimName: campfire-data
          - name: backup-volume
            persistentVolumeClaim:
              claimName: campfire-backups
          restartPolicy: OnFailure
```

## Manual Backup Procedures

### Basic Backup Commands

```bash
# Full backup with compression and verification
./scripts/backup-enhanced.sh full --compress --verify

# Incremental backup (requires WAL mode)
./scripts/backup-enhanced.sh incremental --compress

# Schema-only backup
./scripts/backup-enhanced.sh schema

# Hot backup (zero downtime)
./scripts/backup-enhanced.sh hot --compress --verify
```

### Docker Container Backup

```bash
# Backup from running container
docker-compose exec campfire /app/scripts/backup-enhanced.sh full --compress --verify

# Backup with custom filename
docker-compose exec campfire /app/scripts/backup-enhanced.sh full --compress --output custom_backup_$(date +%Y%m%d).db

# Copy backup out of container
docker cp campfire:/app/backups/latest_full_backup ./local-backup.db.gz
```

### Pre-deployment Backup

```bash
#!/bin/bash
# scripts/pre-deployment-backup.sh

set -euo pipefail

echo "Creating pre-deployment backup..."

# Create timestamped backup
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="pre_deployment_${TIMESTAMP}.db.gz"

# Perform backup
./scripts/backup-enhanced.sh full --compress --verify --output "$BACKUP_FILE"

# Verify backup integrity
./scripts/restore-enhanced.sh --verify-only "$BACKUP_FILE"

echo "Pre-deployment backup completed: $BACKUP_FILE"
echo "BACKUP_FILE=$BACKUP_FILE" > .deployment-backup
```

## Restore Procedures

### Basic Restore Operations

```bash
# List available backups
./scripts/restore-enhanced.sh --list

# Restore latest backup
./scripts/restore-enhanced.sh latest

# Restore specific backup
./scripts/restore-enhanced.sh campfire_backup_20240101_120000.db.gz

# Dry run (test without executing)
./scripts/restore-enhanced.sh campfire_backup_20240101_120000.db.gz --dry-run

# Verify backup integrity only
./scripts/restore-enhanced.sh --verify-only campfire_backup_20240101_120000.db.gz
```

### Point-in-Time Recovery

```bash
#!/bin/bash
# Point-in-time recovery procedure

TARGET_TIME="2024-01-01 12:00:00"
BACKUP_DIR="./backups"

echo "Performing point-in-time recovery to: $TARGET_TIME"

# Find the latest full backup before target time
FULL_BACKUP=$(find "$BACKUP_DIR" -name "campfire_full_*_backup_*.db*" -newermt "$TARGET_TIME" | sort | head -1)

if [[ -z "$FULL_BACKUP" ]]; then
    echo "No full backup found before target time"
    exit 1
fi

echo "Using full backup: $FULL_BACKUP"

# Restore full backup
./scripts/restore-enhanced.sh "$FULL_BACKUP" --auto-confirm

# Apply incremental backups up to target time
find "$BACKUP_DIR" -name "campfire_incremental_*_backup_*.db*" -newermt "$(stat -c %y "$FULL_BACKUP")" -not -newermt "$TARGET_TIME" | sort | while read -r incremental; do
    echo "Applying incremental backup: $incremental"
    ./scripts/apply-incremental-backup.sh "$incremental"
done

echo "Point-in-time recovery completed"
```

### Cross-Platform Restore

```bash
#!/bin/bash
# Restore backup from different platform/architecture

SOURCE_BACKUP="$1"
TARGET_DATABASE="./data/campfire.db"

echo "Performing cross-platform restore..."

# Verify backup compatibility
./scripts/verify-backup-compatibility.sh "$SOURCE_BACKUP"

# Create temporary extraction directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Extract and convert backup if needed
if [[ "$SOURCE_BACKUP" == *.gz ]]; then
    gunzip -c "$SOURCE_BACKUP" > "$TEMP_DIR/backup.db"
else
    cp "$SOURCE_BACKUP" "$TEMP_DIR/backup.db"
fi

# Verify database integrity
sqlite3 "$TEMP_DIR/backup.db" "PRAGMA integrity_check;"

# Perform schema migration if needed
./scripts/migrate-schema.sh "$TEMP_DIR/backup.db"

# Copy to target location
cp "$TEMP_DIR/backup.db" "$TARGET_DATABASE"

echo "Cross-platform restore completed"
```

## Disaster Recovery

### Complete System Recovery

```bash
#!/bin/bash
# Complete disaster recovery procedure

set -euo pipefail

RECOVERY_POINT="${1:-latest}"
BACKUP_SOURCE="${2:-s3://your-backup-bucket}"

echo "Starting disaster recovery..."
echo "Recovery point: $RECOVERY_POINT"
echo "Backup source: $BACKUP_SOURCE"

# 1. Prepare system
echo "Preparing system..."
sudo mkdir -p /opt/campfire/{data,logs,backups}
sudo chown -R 1001:1001 /opt/campfire
cd /opt/campfire

# 2. Download application
echo "Downloading application..."
git clone https://github.com/your-org/campfire-on-rust.git .
cp .env.example .env.production

# 3. Restore backup
echo "Restoring backup..."
if [[ "$BACKUP_SOURCE" == s3://* ]]; then
    # Download from S3
    aws s3 sync "$BACKUP_SOURCE" ./backups/
fi

# Find and restore backup
if [[ "$RECOVERY_POINT" == "latest" ]]; then
    BACKUP_FILE=$(find ./backups -name "campfire_*_backup_*.db*" | sort -r | head -1)
else
    BACKUP_FILE="./backups/$RECOVERY_POINT"
fi

echo "Restoring from: $BACKUP_FILE"
./scripts/restore-enhanced.sh "$BACKUP_FILE" --auto-confirm

# 4. Start services
echo "Starting services..."
docker-compose --profile monitoring --profile proxy up -d

# 5. Verify recovery
echo "Verifying recovery..."
sleep 30
curl -f http://localhost:3000/health || {
    echo "Health check failed"
    exit 1
}

# 6. Run post-recovery checks
echo "Running post-recovery checks..."
./scripts/post-recovery-verification.sh

echo "Disaster recovery completed successfully"
```

### Recovery Verification

```bash
#!/bin/bash
# scripts/post-recovery-verification.sh

set -euo pipefail

echo "Running post-recovery verification..."

# Check application health
echo "Checking application health..."
curl -f http://localhost:3000/health/detailed | jq .

# Verify database integrity
echo "Verifying database integrity..."
docker-compose exec campfire sqlite3 /app/data/campfire.db "PRAGMA integrity_check;"

# Check data consistency
echo "Checking data consistency..."
USER_COUNT=$(docker-compose exec -T campfire sqlite3 /app/data/campfire.db "SELECT COUNT(*) FROM users;")
MESSAGE_COUNT=$(docker-compose exec -T campfire sqlite3 /app/data/campfire.db "SELECT COUNT(*) FROM messages;")
ROOM_COUNT=$(docker-compose exec -T campfire sqlite3 /app/data/campfire.db "SELECT COUNT(*) FROM rooms;")

echo "Data summary:"
echo "  Users: $USER_COUNT"
echo "  Messages: $MESSAGE_COUNT"
echo "  Rooms: $ROOM_COUNT"

# Test core functionality
echo "Testing core functionality..."

# Test WebSocket connection
if command -v wscat &> /dev/null; then
    timeout 10 wscat -c ws://localhost:3000/ws -x '{"type":"ping"}' || echo "WebSocket test failed"
fi

# Test search functionality
SEARCH_RESULT=$(curl -s "http://localhost:3000/api/search?q=test" | jq '.results | length')
echo "Search test returned $SEARCH_RESULT results"

# Check monitoring endpoints
echo "Checking monitoring endpoints..."
curl -f http://localhost:3000/metrics > /dev/null && echo "Metrics endpoint OK"

echo "Post-recovery verification completed"
```

## Data Migration

### Database Migration Between Versions

```bash
#!/bin/bash
# scripts/migrate-database-version.sh

OLD_VERSION="$1"
NEW_VERSION="$2"
BACKUP_FILE="$3"

echo "Migrating database from v$OLD_VERSION to v$NEW_VERSION"

# Create migration backup
./scripts/backup-enhanced.sh full --output "pre_migration_v${OLD_VERSION}_to_v${NEW_VERSION}.db.gz"

# Restore source backup
./scripts/restore-enhanced.sh "$BACKUP_FILE" --auto-confirm

# Run version-specific migrations
case "$OLD_VERSION" in
    "1.0")
        echo "Applying v1.0 to v1.1 migrations..."
        sqlite3 ./data/campfire.db < ./migrations/v1.0_to_v1.1.sql
        ;&
    "1.1")
        echo "Applying v1.1 to v1.2 migrations..."
        sqlite3 ./data/campfire.db < ./migrations/v1.1_to_v1.2.sql
        ;&
    *)
        echo "Migration to v$NEW_VERSION completed"
        ;;
esac

# Verify migration
./scripts/verify-database-schema.sh "$NEW_VERSION"

echo "Database migration completed successfully"
```

### SQLite to PostgreSQL Migration

```bash
#!/bin/bash
# scripts/migrate-sqlite-to-postgresql.sh

SQLITE_DB="$1"
POSTGRES_URL="$2"

echo "Migrating from SQLite to PostgreSQL..."

# Export SQLite data
echo "Exporting SQLite data..."
sqlite3 "$SQLITE_DB" <<EOF
.mode insert users
.output users.sql
SELECT * FROM users;

.mode insert rooms
.output rooms.sql
SELECT * FROM rooms;

.mode insert messages
.output messages.sql
SELECT * FROM messages;

.mode insert room_memberships
.output room_memberships.sql
SELECT * FROM room_memberships;
EOF

# Create PostgreSQL schema
echo "Creating PostgreSQL schema..."
psql "$POSTGRES_URL" -f ./migrations/postgresql_schema.sql

# Import data
echo "Importing data to PostgreSQL..."
psql "$POSTGRES_URL" -f users.sql
psql "$POSTGRES_URL" -f rooms.sql
psql "$POSTGRES_URL" -f messages.sql
psql "$POSTGRES_URL" -f room_memberships.sql

# Update sequences
echo "Updating sequences..."
psql "$POSTGRES_URL" <<EOF
SELECT setval('users_id_seq', (SELECT MAX(id) FROM users));
SELECT setval('rooms_id_seq', (SELECT MAX(id) FROM rooms));
SELECT setval('messages_id_seq', (SELECT MAX(id) FROM messages));
EOF

# Verify migration
echo "Verifying migration..."
SQLITE_USER_COUNT=$(sqlite3 "$SQLITE_DB" "SELECT COUNT(*) FROM users;")
POSTGRES_USER_COUNT=$(psql "$POSTGRES_URL" -t -c "SELECT COUNT(*) FROM users;")

if [[ "$SQLITE_USER_COUNT" -eq "$POSTGRES_USER_COUNT" ]]; then
    echo "Migration verification passed"
else
    echo "Migration verification failed: SQLite=$SQLITE_USER_COUNT, PostgreSQL=$POSTGRES_USER_COUNT"
    exit 1
fi

# Cleanup
rm -f users.sql rooms.sql messages.sql room_memberships.sql

echo "SQLite to PostgreSQL migration completed"
```

## Backup Verification

### Automated Backup Testing

```bash
#!/bin/bash
# scripts/test-backup-integrity.sh

BACKUP_FILE="$1"
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

echo "Testing backup integrity: $BACKUP_FILE"

# Extract backup
if [[ "$BACKUP_FILE" == *.gz ]]; then
    gunzip -c "$BACKUP_FILE" > "$TEST_DIR/test.db"
else
    cp "$BACKUP_FILE" "$TEST_DIR/test.db"
fi

# Test database integrity
echo "Checking database integrity..."
sqlite3 "$TEST_DIR/test.db" "PRAGMA integrity_check;" | grep -q "ok" || {
    echo "Database integrity check failed"
    exit 1
}

# Test schema completeness
echo "Checking schema completeness..."
EXPECTED_TABLES=("users" "rooms" "messages" "room_memberships" "sessions")
for table in "${EXPECTED_TABLES[@]}"; do
    sqlite3 "$TEST_DIR/test.db" ".schema $table" > /dev/null || {
        echo "Missing table: $table"
        exit 1
    }
done

# Test data consistency
echo "Checking data consistency..."
USER_COUNT=$(sqlite3 "$TEST_DIR/test.db" "SELECT COUNT(*) FROM users;")
MESSAGE_COUNT=$(sqlite3 "$TEST_DIR/test.db" "SELECT COUNT(*) FROM messages;")

if [[ $USER_COUNT -eq 0 && $MESSAGE_COUNT -gt 0 ]]; then
    echo "Data consistency error: messages without users"
    exit 1
fi

# Test restore capability
echo "Testing restore capability..."
cp "$TEST_DIR/test.db" "$TEST_DIR/restore_test.db"
sqlite3 "$TEST_DIR/restore_test.db" "SELECT COUNT(*) FROM users;" > /dev/null || {
    echo "Restore test failed"
    exit 1
}

echo "Backup integrity test passed"
```

### Backup Monitoring

```bash
#!/bin/bash
# scripts/monitor-backup-health.sh

BACKUP_DIR="./backups"
MAX_AGE_HOURS=25  # Alert if no backup in 25 hours

echo "Monitoring backup health..."

# Check for recent backups
LATEST_BACKUP=$(find "$BACKUP_DIR" -name "campfire_*_backup_*.db*" -mtime -1 | head -1)

if [[ -z "$LATEST_BACKUP" ]]; then
    echo "ALERT: No recent backups found (within 24 hours)"
    # Send alert notification
    curl -X POST "$SLACK_WEBHOOK_URL" -H 'Content-type: application/json' \
        --data '{"text":"🚨 Campfire backup alert: No recent backups found"}'
    exit 1
fi

# Check backup size consistency
RECENT_BACKUPS=($(find "$BACKUP_DIR" -name "campfire_*_backup_*.db*" -mtime -7 | sort -r | head -5))
SIZES=()

for backup in "${RECENT_BACKUPS[@]}"; do
    size=$(stat -c%s "$backup")
    SIZES+=("$size")
done

# Calculate average size
if [[ ${#SIZES[@]} -gt 1 ]]; then
    avg_size=$(( (${SIZES[0]} + ${SIZES[1]} + ${SIZES[2]:-0}) / ${#SIZES[@]} ))
    latest_size=${SIZES[0]}
    
    # Alert if latest backup is significantly smaller (>50% difference)
    if [[ $latest_size -lt $((avg_size / 2)) ]]; then
        echo "ALERT: Latest backup size ($latest_size) is significantly smaller than average ($avg_size)"
        curl -X POST "$SLACK_WEBHOOK_URL" -H 'Content-type: application/json' \
            --data "{\"text\":\"⚠️ Campfire backup size alert: Latest backup unusually small\"}"
    fi
fi

echo "Backup health check completed"
```

## Remote Backup Storage

### AWS S3 Configuration

```bash
# Environment variables for S3 backup
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export CAMPFIRE_S3_BUCKET=campfire-backups-prod
export CAMPFIRE_S3_REGION=us-west-2

# S3 bucket policy for backup access
cat > backup-bucket-policy.json <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
                "AWS": "arn:aws:iam::ACCOUNT:user/campfire-backup"
            },
            "Action": [
                "s3:GetObject",
                "s3:PutObject",
                "s3:DeleteObject"
            ],
            "Resource": "arn:aws:s3:::campfire-backups-prod/*"
        },
        {
            "Effect": "Allow",
            "Principal": {
                "AWS": "arn:aws:iam::ACCOUNT:user/campfire-backup"
            },
            "Action": "s3:ListBucket",
            "Resource": "arn:aws:s3:::campfire-backups-prod"
        }
    ]
}
EOF

# Apply bucket policy
aws s3api put-bucket-policy --bucket campfire-backups-prod --policy file://backup-bucket-policy.json
```

### Google Cloud Storage Configuration

```bash
# Environment variables for GCS backup
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
export CAMPFIRE_GCS_BUCKET=campfire-backups-prod

# Upload backup to GCS
gsutil cp campfire_backup_20240101_120000.db.gz gs://campfire-backups-prod/

# Set lifecycle policy for automatic cleanup
cat > lifecycle.json <<EOF
{
  "rule": [
    {
      "action": {"type": "Delete"},
      "condition": {"age": 90}
    }
  ]
}
EOF

gsutil lifecycle set lifecycle.json gs://campfire-backups-prod
```

### Encrypted Remote Backup

```bash
#!/bin/bash
# scripts/encrypted-remote-backup.sh

BACKUP_FILE="$1"
GPG_RECIPIENT="backup@yourdomain.com"
REMOTE_DESTINATION="s3://campfire-backups-encrypted/"

echo "Creating encrypted remote backup..."

# Encrypt backup
gpg --trust-model always --encrypt -r "$GPG_RECIPIENT" --output "${BACKUP_FILE}.gpg" "$BACKUP_FILE"

# Upload encrypted backup
aws s3 cp "${BACKUP_FILE}.gpg" "$REMOTE_DESTINATION"

# Verify upload
aws s3 ls "$REMOTE_DESTINATION$(basename "${BACKUP_FILE}.gpg")" || {
    echo "Upload verification failed"
    exit 1
}

# Clean up local encrypted file
rm "${BACKUP_FILE}.gpg"

echo "Encrypted remote backup completed"
```

## Best Practices

### Backup Security

1. **Encryption**: Always encrypt backups containing sensitive data
2. **Access Control**: Limit backup access to authorized personnel only
3. **Audit Logging**: Log all backup and restore operations
4. **Secure Transport**: Use encrypted channels for backup transfers

### Testing and Validation

1. **Regular Testing**: Test restore procedures monthly
2. **Automated Verification**: Implement automated backup integrity checks
3. **Documentation**: Keep restore procedures up-to-date
4. **Training**: Train team members on recovery procedures

### Monitoring and Alerting

1. **Backup Monitoring**: Monitor backup success/failure
2. **Size Monitoring**: Alert on unusual backup sizes
3. **Age Monitoring**: Alert on missing recent backups
4. **Storage Monitoring**: Monitor backup storage usage

### Performance Optimization

1. **Compression**: Use compression to reduce backup size
2. **Incremental Backups**: Use incremental backups for frequent snapshots
3. **Parallel Processing**: Use parallel backup processes when possible
4. **Storage Optimization**: Use appropriate storage classes for different backup types

This comprehensive backup and restore guide ensures data protection and business continuity for Campfire deployments of all sizes.