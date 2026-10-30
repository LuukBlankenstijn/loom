{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer

    sqlx-cli

    yt-dlp
    ffmpeg-headless
  ];
}
