{
  description = "Loom System Monorepo";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
      in
      {
        packages = {
          loom-greeter = pkgs.callPackage ./greeter/nix/package.nix {
            inherit craneLib;
            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset =
                pkgs.lib.fileset.intersection
                  (pkgs.lib.fileset.fromSource (
                    pkgs.lib.cleanSourceWith {
                      src = ./.;
                      filter = craneLib.filterCargoSources;
                    }
                  ))
                  (
                    pkgs.lib.fileset.unions [
                      ./greeter
                      ./shared/greeter-dbus
                    ]
                  );
            };
            cargoLock = ./greeter/Cargo.lock;
          };
          loomd = pkgs.callPackage ./station/nix/package.nix {
            inherit craneLib;
            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset =
                pkgs.lib.fileset.intersection
                  (pkgs.lib.fileset.fromSource (
                    pkgs.lib.cleanSourceWith {
                      src = ./.;
                      filter = craneLib.filterCargoSources;
                    }
                  ))
                  (
                    pkgs.lib.fileset.unions [
                      ./station
                      ./shared/greeter-dbus
                      ./gen/rs
                    ]
                  );
            };
            cargoLock = ./station/Cargo.lock;
          };
          loom-station-registration = pkgs.callPackage ./station-registration/nix/package.nix {
            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.unions [
                ./station-registration
                ./gen/ts
                ./shared/map-react
                ./dashboard/package.json
                ./pnpm-workspace.yaml
                ./pnpm-lock.yaml
              ];
            };
          };
        };
      }
    )
    // {
      nixosModules = {
        loom-greeter = import ./greeter/nix/module.nix self;
        loomd = import ./station/nix/module.nix self;

        default =
          { ... }:
          {
            imports = [
              self.nixosModules.loom-greeter
              self.nixosModules.loomd
            ];
          };
      };

      overlays.default = _: prev: {
        inherit (self.packages.${prev.system})
          loom-greeter
          loomd
          loom-station-registration
          ;
      };
    };
}
