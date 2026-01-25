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
    protoc-gen-prost-crate
  ];
}
