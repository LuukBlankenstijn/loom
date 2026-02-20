{
  lib,
  rustPlatform,
  pkg-config,
  wayland,
  libxkbcommon,
  vulkan-loader,
  openssl,
}:

rustPlatform.buildRustPackage {
  pname = "contest-greeter";
  version = "1.1.2";

  src = lib.cleanSource ./..;

  cargoLock = {
    lockFile = ./../Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    wayland
    libxkbcommon
    vulkan-loader
    openssl
  ];

  postFixup = ''
    patchelf --add-rpath ${
      lib.makeLibraryPath [
        wayland
        libxkbcommon
        vulkan-loader
      ]
    } $out/bin/contest-greeter
  '';

  meta = with lib; {
    description = "A greetd greeter for contests with countdown support";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "contest-greeter";
  };
}
