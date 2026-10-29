# Loom

This mono-repo contains all components that make up the Loom system. The Loom system is a system that automates and simplifies tasks surrounding contest machines in Algorithmic programming contest in the ICPC style.

## Features

- Wallpaper library: uploaded images, image URLs and YouTube videos multicast to the stations
- Integration with ICPC spec compliant contest system
  - gets contest and team information from the contest system
  - displays a countdown right before the contest starts and unlocks it after countdown
  - sets the ip of a team in the contest system for auto login
- Drawing a map of the contest area

### Features planned

- assigning teams to a location on the map
- running custom commands on the team machine via the web-ui
- lock and unlock team machines from the web ui

## Components

The system exist out of several components:

- Backend: Rust gRPC server, central brain of the system. Has stream connections to the different clients, keeps some in-memory state about contest machines and stores everything else in a SQLite database
- Greeter: Greetd greeter running on the contest machine
- Station: systemd service running on team machine written in rust. This acts as the bridge between the backend and the greeter. This is done to make sure the greeter still functions when the rest of the system breaks down.
- Dashboard: This is the web ui that is used to manage everything. It is written in typescript with a generated connectrpc client. The map editor is an embedded React component (`@loom/map-react`) using Konva for canvas rendering.
- station-registration: Tauri 2 desktop app that runs on the contest machine. Reuses the shared `@loom/map-react` component to let an operator click their seat and bind the machine's IP to it.
- shared/map-react: React + Konva map rendering library shared between dashboard (edit mode) and station-registration (view mode + seat overlay).

## Running the backend

### NixOS

```nix
{
  inputs.loom.url = "github:LuukBlankenstijn/loom";

  outputs = { nixpkgs, loom, ... }: {
    nixosConfigurations.server = nixpkgs.lib.nixosSystem {
      modules = [
        loom.nixosModules.loom-backend
        {
          services.loom-backend = {
            enable = true;
            openFirewall = true;
            authTokenFile = "/run/agenix/loom-auth-token";
            contestApi = {
              baseUrl = "https://domjudge.example/domjudge";
              username = "admin";
              passwordFile = "/run/agenix/loom-domjudge-password";
            };
          };
        }
      ];
    };
  };
}
```

This runs `loom-backend.service` as the `loom` user, keeps the database at `/var/lib/loom/loom.db` and serves the dashboard, the HTTP API (`/api`), gRPC-web (`/rpc`) and the station gRPC service on one port (`listenAddress`, default `0.0.0.0:8080`).

The dashboard never sends the auth token. With `authTokenFile` set, put the dashboard behind a reverse proxy that authenticates admins, sets an `x-proxy-authorized` header and strips client-supplied copies, and enable `trustedProxyHeader` so the backend accepts that header instead of the token. Anyone who reaches the backend port directly can send the header too, so only enable it when admin traffic cannot bypass the proxy.

### Configuration

`loom-backend <config.toml>` reads:

```toml
listen = "0.0.0.0:8080"
database_path = "/var/lib/loom/loom.db"
dashboard_dir = "/path/to/dashboard/dist"
auth_token_file = "/run/secrets/loom-auth-token"
trusted_proxy_header = false

[contest_api]
base_url = "https://domjudge.example/domjudge"
username = "admin"
password_file = "/run/secrets/domjudge-password"

[video]
multicast_address = "239.255.76.1:5004"
multicast_ttl = 1
multicast_interface = "10.0.0.1"
```

Only `database_path` is required. `auth_token` and `contest_api.password` may be given inline instead of as files. `trusted_proxy_header` (default `false`) lets gRPC requests with an `x-proxy-authorized` header skip the token check. The `[video]` values above are the defaults, except `multicast_interface`, which is unset by default. Without a config argument it reads `/etc/loom/backend.toml`, or else the environment: `DATABASE_PATH` (required), `DASHBOARD_DIR`, `AUTH_TOKEN`, `TRUSTED_PROXY_HEADER`, `DJ_BASE_URL`, `DJ_USERNAME`, `DJ_PASSWORD`, `VIDEO_MULTICAST_ADDRESS`, `VIDEO_MULTICAST_TTL`, `VIDEO_MULTICAST_INTERFACE`.

### YouTube wallpapers

Nothing is downloaded ahead of time. While a YouTube wallpaper is the current wallpaper (the next contest's wallpaper, or else the default), the backend asks `yt-dlp` for the video's URL. `ffmpeg` then reads the video from YouTube, transcodes it to 1920x1080 30 fps H.264 without audio and loops it. The result goes out as MPEG-TS over UDP multicast to `video.multicast_address`. The live transcode costs about 1.3 CPU cores while it runs. When the current wallpaper changes, the backend switches or stops the stream. `GET /api/wallpaper` returns the stream as an `x-wallpaper-stream: udp://group:port` header, which the greeter plays.

- `yt-dlp` and `ffmpeg` must be on the backend's `PATH`. The NixOS module and the Docker image provide them. Keep `yt-dlp` recent, because YouTube changes break old versions.
- The backend needs internet access to YouTube.
- The stations must be able to receive the multicast stream:
  - The default TTL of 1 keeps the stream on the backend's network segment, so stations on other subnets also need multicast routing.
  - The stations' firewalls must accept UDP on the stream port: set `services.greetd.loom-greeter.openFirewall = true`, with `videoStreamPort` matching the port.
  - Docker's bridge network does not carry multicast to the LAN. Run the backend container with `network_mode: host` to stream.

### Moving from Postgres

Loom up to v0.6.0 stored its data in Postgres. Databases from v0.5.0 and v0.6.0 import directly; run v0.6.0 against older ones once first. Stop the backend, then copy everything into a new SQLite file:

```sh
loom-import-postgres postgres://loom:loom@localhost/loom /var/lib/loom/loom.db
```

The target file must not exist yet, and a failed import leaves nothing behind.

On NixOS the service creates an empty database on its first start, so stop it, remove that database and import as root:

```sh
systemctl stop loom-backend
rm -f /var/lib/loom/loom.db*
nix shell github:LuukBlankenstijn/loom#loom-backend -c loom-import-postgres postgres://… /var/lib/loom/loom.db
chown loom:loom /var/lib/loom/loom.db
systemctl start loom-backend
```

With the compose file, the old database stays available behind the `migrate` profile. Import before the first `docker compose up`, then stop it:

```sh
docker compose --profile migrate up -d db
docker compose run --rm --entrypoint loom-import-postgres backend postgres://loom:loom@db/loom /data/loom.db
docker compose --profile migrate stop db
```
