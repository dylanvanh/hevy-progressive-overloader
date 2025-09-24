# ---- Builder ----
FROM rust:alpine AS builder

RUN apk add --no-cache \
    build-base \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    ca-certificates \
    tzdata

WORKDIR /app

# Copy manifests first for better Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Copy sources
COPY src ./src

# Build release (musl)
RUN cargo build --release

# ---- Runtime ----
FROM alpine:3.20 AS runtime

RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    openssl

# Non-root user
RUN adduser -D -H appuser

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/hevy-progressive-overloader /usr/local/bin/hevy-progressive-overloader

# App directories
RUN mkdir -p /app \
    && chown -R appuser:appuser /app \
    && chown appuser:appuser /usr/local/bin/hevy-progressive-overloader

USER appuser

EXPOSE 3005

CMD ["/usr/local/bin/hevy-progressive-overloader"]
