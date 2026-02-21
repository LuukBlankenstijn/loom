{
  description = "A greetd greeter for icpc contests";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.callPackage ./nix/package.nix { };
          contest-greeter = self.packages.${system}.default;
        }
      );

      nixosModules.default = import ./nix/module.nix self;

      # For convenience
      overlays.default = _: prev: {
        contest-greeter = self.packages.${prev.system}.default;
      };
    };
}
