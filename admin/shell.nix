{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
    pkg-config
    wayland
    libxkbcommon
    libGL
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
