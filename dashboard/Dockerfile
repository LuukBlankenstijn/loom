# syntax=docker/dockerfile:1
# ── Stage 1: Build map-editor WASM ──────────────────────────────
FROM rust:1.94-bookworm AS wasm-builder
WORKDIR /src

RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-binstall && \
    cargo binstall trunk --no-confirm && \
    curl -fsSL https://github.com/WebAssembly/binaryen/releases/download/version_123/binaryen-version_123-x86_64-linux.tar.gz \
      | tar -xz -C /usr/local --strip-components=1

COPY gen/rs/ gen/rs/
COPY shared/ shared/
COPY map-editor/ map-editor/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/map-editor/target \
    cd map-editor && trunk build --release && \
    cp -r dist/ /dist/

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
COPY --from=wasm-builder /dist/ map-editor/dist/

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
