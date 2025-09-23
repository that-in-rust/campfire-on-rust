# Multi-stage build for optimized production image
FROM rust:1.75-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -u 1001 campfire

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY templates ./templates
COPY assets ./assets

# Build the application
RUN cargo build --release --bin campfire-on-rust

# Verify the binary was built
RUN ls -la target/release/ && file target/release/campfire-on-rust

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user and directories
RUN useradd -m -u 1001 campfire && \
    mkdir -p /app/data /app/logs /app/backups && \
    chown -R campfire:campfire /app

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/campfire-on-rust /app/campfire-on-rust
RUN chmod +x /app/campfire-on-rust

# Copy migration scripts
COPY scripts/ /app/scripts/
RUN chmod +x /app/scripts/*.sh

# Create health check script
RUN echo '#!/bin/bash\ncurl -f http://localhost:${CAMPFIRE_PORT:-3000}/health || exit 1' > /app/healthcheck.sh && \
    chmod +x /app/healthcheck.sh

# Switch to non-root user
USER campfire

# Set environment variables with secure defaults
ENV CAMPFIRE_HOST=0.0.0.0
ENV CAMPFIRE_PORT=3000
ENV CAMPFIRE_DATABASE_URL=/app/data/campfire.db
ENV CAMPFIRE_LOG_LEVEL=info
ENV CAMPFIRE_LOG_FORMAT=json
ENV CAMPFIRE_LOG_FILE=/app/logs/campfire.log
ENV CAMPFIRE_BACKUP_DIR=/app/backups
ENV CAMPFIRE_DB_WAL_MODE=true
ENV CAMPFIRE_METRICS_ENABLED=true
ENV CAMPFIRE_TRACE_REQUESTS=false
ENV RUST_LOG=campfire_on_rust=info,tower_http=info

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /app/healthcheck.sh

# Volume for persistent data
VOLUME ["/app/data", "/app/logs", "/app/backups"]

# Run the application
CMD ["/app/campfire-on-rust"]