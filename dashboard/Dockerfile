# syntax=docker/dockerfile:1
# ── Build dashboard ────────────────────────────────────────────
FROM node:24-bookworm-slim AS node-builder

RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /src

# pnpm dep caching
COPY pnpm-*.yaml ./
COPY dashboard/package.json dashboard/
COPY gen/ts/package.json gen/ts/
COPY shared/map-react/package.json shared/map-react/
COPY station-registration/package.json station-registration/

RUN pnpm install --frozen-lockfile

# Copy source and build
COPY gen/ts/ gen/ts/
COPY shared/map-react/ shared/map-react/
COPY dashboard/ dashboard/

RUN cd dashboard && pnpm build

# ── Serve with Caddy ───────────────────────────────────────────
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
