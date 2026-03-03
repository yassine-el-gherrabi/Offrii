# ── Dev stage (target for docker-compose.override.yml) ──────────────
FROM rust:slim-bookworm AS dev

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch cargo-chef

WORKDIR /app

# Sources are volume-mounted in dev, not copied
CMD ["cargo", "watch", "-x", "run -p api", "-w", "crates/"]

# ── Planner stage (generate recipe.json for dependency caching) ─────
FROM rust:slim-bookworm AS planner

RUN cargo install cargo-chef

WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Cook stage (build dependencies from recipe — cached layer) ──────
FROM rust:slim-bookworm AS cook

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ── Builder stage (compile the actual application) ──────────────────
FROM cook AS builder

COPY . .
RUN cargo build --release --bin api

# ── Runtime stage (minimal production image) ────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

RUN groupadd --system app && useradd --system --gid app app

COPY --from=builder /app/target/release/api /usr/local/bin/api

USER app

EXPOSE 3000

CMD ["api"]
