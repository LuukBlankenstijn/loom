{
  lib,
  craneLib,
  src,
  cargoLock,
  pkg-config,
  makeWrapper,
  wayland,
  libxkbcommon,
  vulkan-loader,
  openssl,
  glib,
  gst_all_1,
}:
let
  gstPlugins = with gst_all_1; [
    gstreamer
    gst-plugins-base
    gst-plugins-good
    gst-plugins-bad
    gst-libav
  ];

  commonArgs = {
    inherit src;
    pname = "loom-greeter";
    version = "1.1.2";

    cargoRoot = "greeter";
    inherit cargoLock;

    postUnpack = ''
      cd $sourceRoot/greeter
      sourceRoot=.
    '';

    nativeBuildInputs = [ pkg-config ];
    buildInputs = [
      wayland
      libxkbcommon
      vulkan-loader
      openssl
      glib
      gst_all_1.gstreamer
      gst_all_1.gst-plugins-base
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ makeWrapper ];

    postFixup = ''
      patchelf --add-rpath ${
        lib.makeLibraryPath [
          wayland
          libxkbcommon
          vulkan-loader
        ]
      } $out/bin/loom-greeter
      wrapProgram $out/bin/loom-greeter \
        --prefix GST_PLUGIN_SYSTEM_PATH_1_0 : ${lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0" gstPlugins}
    '';

    meta = with lib; {
      description = "A greetd greeter for contests with countdown support";
      license = licenses.mit;
      platforms = platforms.linux;
      mainProgram = "loom-greeter";
    };
  }
)
