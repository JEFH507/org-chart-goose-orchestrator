# Builder
FROM rust:1.81-alpine3.20 AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /app
# Pre-cache deps
COPY Cargo.toml ./
COPY src/controller/Cargo.toml src/controller/Cargo.toml
RUN mkdir -p src/controller/src && echo 'fn main(){}' > src/controller/src/main.rs && cargo build -p goose-controller --release || true
# Build actual
COPY src/controller/src src/controller/src
RUN cargo build -p goose-controller --release

# Runtime
FROM alpine:3.20
RUN apk add --no-cache ca-certificates curl && adduser -D -H -u 10001 appuser
WORKDIR /home/appuser
COPY --from=builder /app/target/release/goose-controller /usr/local/bin/goose-controller
USER appuser
EXPOSE 8088
ENV CONTROLLER_PORT=8088
ENTRYPOINT ["/usr/local/bin/goose-controller"]
