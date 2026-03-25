{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    rustup
    trunk
    gcc
    pkg-config
    wayland
    libxkbcommon
    vulkan-loader
  ];

  shellHook = ''
    export RUSTUP_HOME=$PWD/.rustup
    export CARGO_HOME=$PWD/.cargo
    export PATH=$CARGO_HOME/bin:$PATH

    export CC="${pkgs.gcc}/bin/gcc"

    if [ ! -d "$RUSTUP_HOME" ]; then
      rustup default stable
      rustup target add wasm32-unknown-unknown
    fi
  '';

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
