# Builder (workspace, full context)
# Use nightly to support edition2024 dependencies
FROM rustlang/rust:nightly-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build -p goose-controller --release

# Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /home/appuser
COPY --from=builder /app/target/release/goose-controller /usr/local/bin/goose-controller
USER nobody
EXPOSE 8088
ENV CONTROLLER_PORT=8088
ENTRYPOINT ["/usr/local/bin/goose-controller"]
