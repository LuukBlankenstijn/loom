{
  lib,
  stdenv,
  pnpm_11,
  pnpmConfigHook,
  fetchPnpmDeps,
  nodejs,
  src,
}:

stdenv.mkDerivation (finalAttrs: {
  pname = "loom-dashboard";
  version = "0.1.0";

  inherit src;

  pnpmDeps = fetchPnpmDeps {
    inherit (finalAttrs) pname version src;
    pnpmRoot = ".";
    fetcherVersion = 4;
    hash = "sha256-SxJ82w5IHyZVBnSxurEkGVUMjrhFAWqDDxEcVBCrc1Y=";
  };

  nativeBuildInputs = [
    pnpmConfigHook
    pnpm_11
    nodejs
  ];

  buildPhase = ''
    runHook preBuild
    pnpm --filter dashboard run build
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    cp -r dashboard/dist $out
    runHook postInstall
  '';

  meta = {
    description = "Loom admin dashboard single-page application";
    license = lib.licenses.mit;
    platforms = lib.platforms.all;
  };
})
