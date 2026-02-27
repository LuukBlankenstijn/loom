{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    buf
    protoc-gen-go
    protoc-gen-connect-go
    protoc-gen-tonic
    protoc-gen-prost
    protoc-gen-es
    protoc-gen-prost-crate

    nodejs
    pnpm

    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
  ];
}
