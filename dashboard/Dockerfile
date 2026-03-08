# ── Stage 1: Build map-editor WASM ───────────────────────────────
FROM rust:1.94-bookworm AS wasm-builder

RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-binstall && \
    cargo binstall trunk --no-confirm && \
    curl -fsSL https://github.com/WebAssembly/binaryen/releases/download/version_123/binaryen-version_123-x86_64-linux.tar.gz \
      | tar -xz -C /usr/local --strip-components=1

WORKDIR /src

# Cargo dep caching with dummy sources
COPY map-editor/Cargo.toml map-editor/Cargo.lock map-editor/
COPY gen/rs/Cargo.toml gen/rs/
COPY shared/map/Cargo.toml shared/map/

RUN mkdir -p map-editor/src gen/rs/src shared/map/src && \
    echo "fn main() {}" > map-editor/src/main.rs && \
    touch gen/rs/src/lib.rs && \
    touch shared/map/src/lib.rs

RUN cd map-editor && cargo build --target wasm32-unknown-unknown --release 2>/dev/null || true

# Copy real sources and build WASM
COPY gen/rs/ gen/rs/
COPY shared/map/ shared/map/
COPY map-editor/src/ map-editor/src/
COPY map-editor/fonts/ map-editor/fonts/
COPY map-editor/index.html map-editor/

# Force cargo to detect changed sources (Docker COPY preserves original timestamps)
RUN touch gen/rs/src/lib.rs shared/map/src/lib.rs map-editor/src/main.rs
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

# ── Stage 3: Serve with Caddy ────────────────────────────────────
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
