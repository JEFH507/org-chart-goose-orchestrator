# Multi-stage build for privacy-guard-proxy

# Build stage
FROM rust:1.83 as builder

WORKDIR /build

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /build/target/release/privacy-guard-proxy /app/privacy-guard-proxy

# Expose the proxy port
EXPOSE 8090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8090/api/status || exit 1

# Run the binary
CMD ["/app/privacy-guard-proxy"]
