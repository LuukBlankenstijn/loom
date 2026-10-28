{
  lib,
  craneLib,
  src,
  cargoLock,
}:
let
  commonArgs = {
    inherit src;
    pname = "loom-backend";
    version = "0.1.0";
    strictDeps = true;

    cargoRoot = "backend";
    inherit cargoLock;

    postUnpack = ''
      cd $sourceRoot/backend
      sourceRoot=.
    '';

    SQLX_OFFLINE = "true";
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    meta = with lib; {
      description = "Loom backend serving the admin API, the station gRPC service and the dashboard.";
      license = licenses.mit;
      platforms = platforms.linux;
      mainProgram = "loom-backend";
    };
  }
)
