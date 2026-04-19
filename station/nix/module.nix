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

    authTokenCommand = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "The command to the the authentication token for the service.";
    };

    includeSystemPackages = mkEnableOption "Put all system packages on the path of the loomd service. Allows to use systempackages when running remote commands";
  };

  config = mkIf cfg.enable {
    systemd.services.loomd = {
      description = "Loom Daemon Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      path = lib.optionals cfg.includeSystemPackages config.environment.systemPackages;
      serviceConfig = {
        ExecStart =
          let
            authFlag = lib.optionalString (
              cfg.authTokenCommand != null
            ) "--auth-command=${lib.escapeShellArg cfg.authTokenCommand}";
          in
          pkgs.writeShellScript "start-loomd" ''
            exec ${stationPackage}/bin/loomd --server ${lib.escapeShellArg cfg.server} ${authFlag}
          '';
        Restart = "on-failure";
        RestartSec = "5s";
        User = "root";
        Type = "exec";
      };
    };
  };
}
