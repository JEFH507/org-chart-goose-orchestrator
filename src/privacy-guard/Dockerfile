# Multi-stage Dockerfile for Privacy Guard Service
# Stage 1: Builder - Compile Rust application
# Stage 2: Runtime - Minimal Debian image with compiled binary

# ============================================================================
# STAGE 1: Builder
# ============================================================================
FROM rust:1.83-bookworm AS builder

WORKDIR /build

# Copy workspace manifests
COPY Cargo.toml Cargo.lock* ./

# Copy actual source code
COPY src ./src
COPY tests ./tests

# Build the release binary
RUN cargo build --release

# Verify binary was created
RUN ls -lh target/release/privacy-guard

# ============================================================================
# STAGE 2: Runtime
# ============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies
# - ca-certificates: For HTTPS (future Vault integration)
# - curl: For healthcheck
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean

# Create config directory
RUN mkdir -p /etc/guard-config

# Copy compiled binary from builder
COPY --from=builder /build/target/release/privacy-guard /usr/local/bin/privacy-guard

# Make binary executable
RUN chmod +x /usr/local/bin/privacy-guard

# Create non-root user for security
RUN useradd -r -s /bin/false -u 1000 guarduser && \
    chown -R guarduser:guarduser /etc/guard-config

# Switch to non-root user
USER guarduser

# Expose guard service port
EXPOSE 8089

# Environment variables (defaults, overridable)
ENV GUARD_PORT=8089 \
    GUARD_MODE=MASK \
    GUARD_CONFIDENCE=MEDIUM \
    RUST_LOG=info \
    CONFIG_PATH=/etc/guard-config

# Healthcheck: Verify service is responding
# Runs every 10s, 3s timeout, 3 retries before unhealthy
HEALTHCHECK --interval=10s --timeout=3s --retries=3 --start-period=5s \
    CMD curl -f http://localhost:8089/status || exit 1

# Run the guard service
CMD ["privacy-guard"]
