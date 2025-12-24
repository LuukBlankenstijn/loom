#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root (e.g. sudo ./install.sh)."
  exit 1
fi

if command -v apt-get >/dev/null 2>&1; then
  apt-get update
  apt-get install -y \
    ca-certificates \
    curl \
    tar \
    libgtk-4-1 \
    libglib2.0-0 \
    liblightdm-gobject-1-0
else
  echo "Unsupported package manager. Install curl + tar and runtime libs for gtk4, glib, lightdm."
  exit 1
fi

REPO_SLUG="${REPO_SLUG:-LuukBlankenstijn/LightDM-Contest-Greeter}"
ASSET_NAME="${ASSET_NAME:-lightdm-contest-greeter-linux-x86_64.tar.gz}"
API_URL="https://api.github.com/repos/${REPO_SLUG}/releases/latest"

release_json="$(curl -fsSL "$API_URL")"
if command -v python3 >/dev/null 2>&1; then
  download_url="$(printf '%s' "$release_json" | python3 - <<'PY'
import json, os, sys
data = json.load(sys.stdin)
asset_name = os.environ.get("ASSET_NAME")
for asset in data.get("assets", []):
    if asset.get("name") == asset_name:
        print(asset.get("browser_download_url", ""))
        break
PY
)"
else
  download_url="$(printf '%s' "$release_json" | tr -d '\n' | sed -n "s/.*\\\"browser_download_url\\\":\\\"\\([^\\\"]*${ASSET_NAME}[^\\\"]*\\)\\\".*/\\1/p")"
fi

if [ -z "$download_url" ]; then
  echo "Failed to find release asset ${ASSET_NAME} in ${REPO_SLUG}."
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
curl -fsSL "$download_url" -o "$tmp_dir/$ASSET_NAME"
tar -xzf "$tmp_dir/$ASSET_NAME" -C "$tmp_dir"

install -m 0755 "$tmp_dir/lightdm-contest-greeter" /usr/local/bin/

install -d /etc/dbus-1/system.d
cat > /etc/dbus-1/system.d/nl.luukblankenstijn.ContestGreeterService.conf <<'EOF'
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <policy user="lightdm">
    <allow own="nl.luukblankenstijn.ContestGreeterService"/>
  </policy>
  <policy context="default">
    <allow send_destination="nl.luukblankenstijn.ContestGreeterService"/>
  </policy>
</busconfig>
EOF

install -d /usr/share/xgreeters
cat > /usr/share/xgreeters/lightdm-contest-greeter.desktop <<'EOF'
[Desktop Entry]
Name=Contest Greeter
Comment=LightDM contest greeter
Exec=lightdm-contest-greeter
Type=Application
X-LightDM-DesktopName=ContestGreeter
EOF

install -d /etc/lightdm/lightdm.conf.d
cat > /etc/lightdm/lightdm.conf.d/50-contest-greeter.conf <<'EOF'
[Seat:*]
greeter-session=lightdm-contest-greeter
EOF

if [ ! -f /etc/lightdm/lightdm-contest-greeter.conf ]; then
  cat > /etc/lightdm/lightdm-contest-greeter.conf <<'EOF'
log_level = "info"
enable_dbus = true
chain = "chain"
countdown_from = 10
countdown_end_login = true
countdown_label_color = "white"
interval = 3

# background_source = "/path/to/wallpaper.jpg"
# countdown_end_time = "2025-12-24 20:00:00"
# session = "your-session"
# username = "team"
# password = "password"
# url = "https://contest.example/api"
EOF
fi

echo "Installed lightdm-contest-greeter. Update /etc/lightdm/lightdm-contest-greeter.conf as needed."
