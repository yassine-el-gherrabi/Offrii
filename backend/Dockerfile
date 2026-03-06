# ── Builder ──────────────────────────────────────────
FROM rust:1.93-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY rest-api/Cargo.toml rest-api/Cargo.toml

# Cache dependencies via dummy build
RUN mkdir -p rest-api/src rest-api/src/bin \
    && echo "fn main() {}" > rest-api/src/main.rs \
    && echo "fn main() {}" > rest-api/src/bin/migrate.rs \
    && echo "" > rest-api/src/lib.rs \
    && cargo build --release --workspace 2>&1 || true \
    && rm -rf rest-api/src

COPY rest-api/ rest-api/
RUN touch rest-api/src/main.rs rest-api/src/lib.rs rest-api/src/bin/migrate.rs
RUN cargo build --release --workspace

# ── Runtime ──────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 appuser \
    && useradd --uid 1000 --gid appuser --shell /bin/false appuser

COPY --from=builder /build/target/release/rest-api /usr/local/bin/rest-api
COPY --from=builder /build/target/release/migrate  /usr/local/bin/migrate
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

WORKDIR /app
USER appuser
EXPOSE 3000

HEALTHCHECK --interval=10s --timeout=5s --retries=5 --start-period=30s \
    CMD curl -sf http://localhost:3000/health/live || exit 1

ENTRYPOINT ["docker-entrypoint.sh"]
