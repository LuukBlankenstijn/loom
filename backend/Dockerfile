# syntax=docker/dockerfile:1
# ── Build ────────────────────────────────────────────────────────
FROM rust:1.94-bookworm AS builder
WORKDIR /src

ENV SQLX_OFFLINE=true

COPY gen/rs/ gen/rs/
COPY shared/ shared/
COPY backend/ backend/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/backend/target \
    cd backend && cargo build --release && \
    cp target/release/loom-backend /loom-backend

# ── Runtime ──────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /loom-backend /usr/local/bin/loom-backend

EXPOSE 8080

ENTRYPOINT ["loom-backend"]
