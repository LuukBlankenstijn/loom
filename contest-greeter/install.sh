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

ASSET_NAME="${ASSET_NAME:-lightdm-contest-greeter-linux-x86_64.tar.gz}"
DOWNLOAD_URL="${DOWNLOAD_URL:-https://github.com/LuukBlankenstijn/LightDM-Contest-Greeter/releases/latest/download/${ASSET_NAME}}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
curl -fsSL "$DOWNLOAD_URL" -o "$tmp_dir/$ASSET_NAME"
tar -xzf "$tmp_dir/$ASSET_NAME" -C "$tmp_dir"

install -m 0755 "$tmp_dir/lightdm-contest-greeter" /usr/local/bin/

install -d /etc/dbus-1/system.d
cat >/etc/dbus-1/system.d/nl.luukblankenstijn.ContestGreeterService.conf <<'EOF'
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
cat >/usr/share/xgreeters/lightdm-contest-greeter.desktop <<'EOF'
[Desktop Entry]
Name=Contest Greeter
Comment=LightDM contest greeter
Exec=lightdm-contest-greeter
Type=Application
X-LightDM-DesktopName=ContestGreeter
EOF

install -d /etc/lightdm/lightdm.conf.d
cat >/etc/lightdm/lightdm.conf.d/50-contest-greeter.conf <<'EOF'
[Seat:*]
greeter-session=lightdm-contest-greeter
EOF

if [ ! -f /etc/lightdm/lightdm-contest-greeter.conf ]; then
    cat >/etc/lightdm/lightdm-contest-greeter.conf <<'EOF'
log_level = "info"
enable_dbus = true

# background_source = "/path/to/wallpaper.jpg"
# username = "team"
# password = "password"
EOF
fi

echo "Installed lightdm-contest-greeter. Update /etc/lightdm/lightdm-contest-greeter.conf as needed."
