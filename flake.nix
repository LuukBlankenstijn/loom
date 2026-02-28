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

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = craneLib.filterCargoSources;
        };
      in
      {
        packages = {
          loom-greeter = pkgs.callPackage ./greeter/nix/package.nix {
            inherit craneLib src;
            cargoLock = ./greeter/Cargo.lock;
          };
          loomd = pkgs.callPackage ./station/nix/package.nix {
            inherit craneLib src;
            cargoLock = ./station/Cargo.lock;
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
        inherit (self.packages.${prev.system}) loom-greeter loomd;
      };
    };
}
