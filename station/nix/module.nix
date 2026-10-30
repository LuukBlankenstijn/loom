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
      default = flake.packages.${pkgs.stdenv.hostPlatform.system}.loomd;
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

    registrationTool = mkOption {
      type = types.nullOr types.package;
      default = flake.packages.${pkgs.stdenv.hostPlatform.system}.loom-station-registration;
      description = ''
        The loom-station-registration package whose binary loomd launches into
        the active graphical session (greeter or desktop) when it receives a
        start-registration-tool command. Set to null to fall back to PATH
        resolution of `loom-station-registration`.
      '';
    };

    enableRegistrationTool = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Whether loomd launches the registration tool on request. Disable on
        machines that are not team stations.
      '';
    };

    label = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "judgehost";
      description = ''
        Free-form label loomd reports for this station, e.g. "team",
        "judgehost" or "service". It is trimmed and lowercased; an empty
        string means no label. Admins can override it in the dashboard.
      '';
    };

    greeter.enable = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Whether loomd integrates with the loom greeter. When disabled, loomd
        does not connect to the greeter over dbus, always reports the station
        as logged in and ignores logout requests.
      '';
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
            registrationToolFlag = lib.optionalString (
              cfg.registrationTool != null
            ) "--registration-tool ${lib.escapeShellArg (lib.getExe cfg.registrationTool)}";
            labelFlag = lib.optionalString (cfg.label != null) "--label=${lib.escapeShellArg cfg.label}";
            greeterFlag = lib.optionalString (!cfg.greeter.enable) "--no-greeter";
            noRegistrationToolFlag = lib.optionalString (!cfg.enableRegistrationTool) "--no-registration-tool";
          in
          pkgs.writeShellScript "start-loomd" ''
            exec ${stationPackage}/bin/loomd --server ${lib.escapeShellArg cfg.server} ${authFlag} ${registrationToolFlag} ${labelFlag} ${greeterFlag} ${noRegistrationToolFlag}
          '';
        Restart = "on-failure";
        RestartSec = "5s";
        User = "root";
        Type = "exec";
      };
    };
  };
}
