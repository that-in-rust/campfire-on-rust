# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 campfire

# Copy the binary
COPY --from=builder /app/target/release/campfire-rust /usr/local/bin/campfire-rust

# Set ownership and permissions
RUN chown campfire:campfire /usr/local/bin/campfire-rust
RUN chmod +x /usr/local/bin/campfire-rust

# Switch to app user
USER campfire

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["/usr/local/bin/campfire-rust"]