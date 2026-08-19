{
  lib,
  stdenv,
  rustPlatform,
  cargo,
  rustc,
  cargo-tauri,
  pnpm_11,
  pnpmConfigHook,
  fetchPnpmDeps,
  nodejs,
  pkg-config,
  wrapGAppsHook4,
  gobject-introspection,
  webkitgtk_4_1,
  gtk3,
  libsoup_3,
  librsvg,
  glib,
  cairo,
  pango,
  gdk-pixbuf,
  atkmm,
  openssl,
  dbus,
  harfbuzz,
  src,
}:

stdenv.mkDerivation (finalAttrs: {
  pname = "loom-station-registration";
  version = "0.1.0";

  inherit src;

  cargoDeps = rustPlatform.fetchCargoVendor {
    inherit (finalAttrs) pname version src;
    sourceRoot = "${finalAttrs.src.name}/station-registration/src-tauri";
    hash = "sha256-ALITVjyzxjOi6sNO4sWxyVohyA9seBB/IV0JUyynRaU=";
  };

  pnpmDeps = fetchPnpmDeps {
    inherit (finalAttrs) pname version src;
    pnpmRoot = ".";
    fetcherVersion = 4;
    hash = "sha256-SxJ82w5IHyZVBnSxurEkGVUMjrhFAWqDDxEcVBCrc1Y=";
  };

  nativeBuildInputs = [
    rustPlatform.cargoSetupHook
    cargo
    rustc
    cargo-tauri.hook
    pnpmConfigHook
    pnpm_11
    nodejs
    pkg-config
    wrapGAppsHook4
    gobject-introspection
  ];

  buildInputs = [
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

  cargoRoot = "station-registration/src-tauri";
  buildAndTestSubdir = "station-registration/src-tauri";

  meta = {
    description = "Loom station registration desktop application";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "loom-station-registration";
  };
})
