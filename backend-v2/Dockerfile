FROM rust:1.94-bookworm AS builder

ENV SQLX_OFFLINE=true

WORKDIR /src

# Cargo dep caching with dummy sources
COPY backend/Cargo.toml backend/Cargo.lock backend/
COPY gen/rs/Cargo.toml gen/rs/

RUN mkdir -p backend/src gen/rs/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    touch gen/rs/src/lib.rs

RUN cd backend && cargo build --release 2>/dev/null || true

# Copy real sources and build
COPY gen/rs/ gen/rs/
COPY backend/src/ backend/src/
COPY backend/migrations/ backend/migrations/
COPY backend/.sqlx/ backend/.sqlx/

RUN touch gen/rs/src/lib.rs backend/src/main.rs
RUN cd backend && cargo build --release

# ── Runtime ──────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/backend/target/release/loom-backend /usr/local/bin/loom-backend

EXPOSE 8080

ENTRYPOINT ["loom-backend"]
