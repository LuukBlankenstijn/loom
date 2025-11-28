{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.rustfmt
          pkgs.clippy
          pkgs.rust-analyzer
          pkgs.pkg-config
          pkgs.lightdm
          pkgs.glib.dev
          pkgs.glib.out
          pkgs.gtk4
        ];
      };
    };
}
