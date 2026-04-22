# syntax=docker/dockerfile:1
# ── cargo-chef base ─────────────────────────────────────────────
FROM lukemathwalker/cargo-chef:0.1.77-rust-1.94-bookworm AS chef
WORKDIR /src

FROM chef AS planner
COPY . .
RUN cd map-editor && cargo chef prepare --recipe-path /recipe.json

# ── Stage 1: Build map-editor WASM ──────────────────────────────
FROM chef AS wasm-builder

RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-binstall && \
    cargo binstall trunk --no-confirm && \
    curl -fsSL https://github.com/WebAssembly/binaryen/releases/download/version_123/binaryen-version_123-x86_64-linux.tar.gz \
      | tar -xz -C /usr/local --strip-components=1

# Copy Cargo.toml files and stub sources for path dependencies
COPY map-editor/Cargo.toml map-editor/Cargo.toml
COPY shared/client/Cargo.toml shared/client/Cargo.toml
COPY shared/map/Cargo.toml shared/map/Cargo.toml
COPY shared/core/Cargo.toml shared/core/Cargo.toml
COPY shared/proto-bridge/Cargo.toml shared/proto-bridge/Cargo.toml
COPY gen/rs/Cargo.toml gen/rs/Cargo.toml
RUN mkdir -p shared/client/src shared/map/src shared/core/src shared/proto-bridge/src gen/rs/src && \
    touch shared/client/src/lib.rs shared/map/src/lib.rs shared/core/src/lib.rs shared/proto-bridge/src/lib.rs gen/rs/src/lib.rs

# Cook dependencies from recipe (layer is cached unless deps change)
COPY --from=planner /recipe.json map-editor/recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cd map-editor && cargo chef cook --release --target wasm32-unknown-unknown --recipe-path recipe.json

# Copy real sources and build WASM
COPY gen/rs/ gen/rs/
COPY shared/ shared/
COPY map-editor/src/ map-editor/src/
COPY map-editor/fonts/ map-editor/fonts/
COPY map-editor/index.html map-editor/

RUN cd map-editor && trunk build --release

# ── Stage 2: Build dashboard ────────────────────────────────────
FROM node:24-bookworm-slim AS node-builder

RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /src

# pnpm dep caching
COPY pnpm-workspace.yaml pnpm-lock.yaml ./
COPY dashboard/package.json dashboard/
COPY gen/ts/package.json gen/ts/

RUN pnpm install --frozen-lockfile

# Copy source and build
COPY gen/ts/ gen/ts/
COPY dashboard/ dashboard/
COPY --from=wasm-builder /src/map-editor/dist/ map-editor/dist/

RUN cd dashboard && pnpm build

# ── Stage 3: Serve with Caddy ───────────────────────────────────
FROM caddy:alpine

COPY <<'EOF' /etc/caddy/Caddyfile
:80 {
	root * /srv

	# SPA fallback (try directory index.html before falling back to dashboard SPA)
	try_files {path} {path}/index.html /index.html

	file_server
}
EOF

COPY --from=node-builder /src/dashboard/dist/ /srv/

EXPOSE 80
