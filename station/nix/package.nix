{
  lib,
  craneLib,
  src,
  cargoLock,
  pkg-config,
  openssl,
  dbus,
  systemd,
  protobuf,
}:
let
  commonArgs = {
    inherit src;
    pname = "loomd";
    version = "0.1.0";

    cargoRoot = "station";
    inherit cargoLock;

    postUnpack = ''
      cd $sourceRoot/station
      sourceRoot=.
    '';

    nativeBuildInputs = [
      pkg-config
      protobuf
    ];
    buildInputs = [
      openssl
      dbus
      systemd
      protobuf
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    postFixup = ''
      patchelf --add-rpath ${
        lib.makeLibraryPath [
          openssl
          dbus
          systemd
          protobuf
        ]
      } $out/bin/loomd
    '';

    meta = with lib; {
      description = "A service to control the greeter and other parts of the loom system.";
      license = licenses.mit;
      platforms = platforms.linux;
      mainProgram = "loomd";
    };
  }
)
