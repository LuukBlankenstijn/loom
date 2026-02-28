flake:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.loomd;
  inherit (lib)
    mkEnableOption
    mkOption
    types
    mkIf
    ;

  stationPackage = cfg.package;
in
{
  options.services.loomd = {
    enable = mkEnableOption "loomd, a service to control the greeter and other parts of the loom system.";

    package = mkOption {
      type = types.package;
      default = flake.packages.${pkgs.system}.loomd;
      description = "The loomd package to use.";
    };

    server = mkOption {
      type = types.str;
      description = "The server to connect to.";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.loomd = {
      description = "Loom Daemon Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      serviceConfig = {
        ExecStart = "${stationPackage}/bin/loomd --server ${cfg.server}";
        Restart = "on-failure";
        RestartSec = "5s";
        User = "root";
        Type = "exec";
      };
    };
  };
}
