{
  pkgs ? import <nixpkgs> { },
}:

let
  parent = import ../shell.nix { inherit pkgs; };

  libs = with pkgs; [
    webkitgtk_4_1
    gtk3
    libsoup_3
    librsvg
    glib
    cairo
    pango
    gdk-pixbuf
    atkmm
    openssl
    dbus
    harfbuzz
  ];
in
pkgs.mkShell {
  inputsFrom = [ parent ];

  nativeBuildInputs = with pkgs; [
    pkg-config
    gobject-introspection
    cargo-tauri
  ];

  buildInputs = libs;

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" libs}:''${PKG_CONFIG_PATH:-}"
    export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:''${XDG_DATA_DIRS:-}"
  '';
}
