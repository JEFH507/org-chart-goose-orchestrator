#!/bin/bash
# Build controller image from project root with full workspace context

cd "$(dirname "$0")"

echo "Building goose-controller Docker image..."
docker build \
  -t ghcr.io/jefh507/goose-controller:latest \
  -f - . << 'DOCKERFILE'
# Builder (workspace, full context)
FROM rustlang/rust:nightly-bookworm AS builder
WORKDIR /app
COPY . .
WORKDIR /app/src/controller
RUN cargo build --release

# Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /home/appuser
COPY --from=builder /app/target/release/goose-controller /usr/local/bin/goose-controller
USER nobody
EXPOSE 8088
ENV CONTROLLER_PORT=8088
ENTRYPOINT ["/usr/local/bin/goose-controller"]
DOCKERFILE

echo "Build complete!"
