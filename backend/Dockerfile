# syntax=docker/dockerfile:1
FROM lukemathwalker/cargo-chef:0.1.77-rust-1.94-bookworm AS chef
WORKDIR /src

FROM chef AS planner
COPY . .
RUN cd backend && cargo chef prepare --recipe-path /recipe.json

# ── Build ────────────────────────────────────────────────────────
FROM chef AS builder

ENV SQLX_OFFLINE=true

# Cook dependencies from recipe (layer is cached unless deps change)
COPY --from=planner /recipe.json backend/recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cd backend && cargo chef cook --release --recipe-path recipe.json

# Copy real sources over stubs and build
COPY gen/rs/ gen/rs/
COPY shared/ shared/
COPY backend/src/ backend/src/
COPY backend/migrations/ backend/migrations/
COPY backend/.sqlx/ backend/.sqlx/

RUN cd backend && cargo build --release

# ── Runtime ──────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/backend/target/release/loom-backend /usr/local/bin/loom-backend

EXPOSE 8080

ENTRYPOINT ["loom-backend"]
