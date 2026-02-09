flake:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.greetd.contest-greeter;
  inherit (lib)
    mkEnableOption
    mkOption
    types
    mkIf
    ;

  tomlFormat = pkgs.formats.toml { };

  greeterConfig = {
    log_level = cfg.logLevel;
    enable_dbus = cfg.enableDbus;
    chain = cfg.chain;
    background_source = cfg.backgroundSource;
    session = cfg.session;
    username = cfg.username;
    password = cfg.password;
    url = cfg.url;
  };

  # Filter out null values
  filteredConfig = lib.filterAttrs (_: v: v != null) greeterConfig;

  configFile = tomlFormat.generate "contest-greeter.toml" filteredConfig;

  greeterPackage = flake.packages.${pkgs.system}.default;
in
{
  options.services.greetd.contest-greeter = {
    enable = mkEnableOption "contest-greeter, a greetd greeter for icpc contests";

    package = mkOption {
      type = types.package;
      default = greeterPackage;
      description = "The contest-greeter package to use.";
    };

    cagePackage = mkOption {
      type = types.package;
      default = pkgs.cage;
      description = "The cage package to use";
    };

    logLevel = mkOption {
      type = types.str;
      default = "info";
      description = "Log level (env_logger style, e.g. info, debug).";
    };

    enableDbus = mkOption {
      type = types.bool;
      default = true;
      description = "Enable or disable the dbus module.";
    };

    chain = mkOption {
      type = types.str;
      default = "chain";
      description = "Key sequence to toggle the login UI.";
    };

    backgroundSource = mkOption {
      type = types.nullOr (
        types.oneOf [
          types.str
          types.path
        ]
      );
      default = null;
      description = "File path or URL for the background image.";
      example = "/etc/greetd/background.png";
    };

    session = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Session to start after login.";
      example = "gnome-session";
    };

    username = mkOption {
      type = types.str;
      default = "";
      description = "Username for automatic login.";
    };

    password = mkOption {
      type = types.str;
      default = "";
      description = "Password for automatic login.";
    };

    url = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Contest API URL returning JSON with start_time (RFC3339).";
      example = "https://api.example.com/contest";
    };
  };

  config = mkIf cfg.enable (
    lib.mkMerge [
      {
        services.greetd = {
          enable = true;
          settings.default_session = {
            command = "${cfg.cagePackage}/bin/cage -s -- ${pkgs.systemd}/bin/systemd-cat -t contest-greeter ${cfg.package}/bin/contest-greeter ${configFile}";
            user = "greeter";
          };
        };
      }
      # Only add D-Bus if enabled
      (mkIf cfg.enableDbus {
        services.dbus.packages = [
          (pkgs.writeTextDir "share/dbus-1/system.d/nl.luukblankenstijn.ContestGreeterService.conf" ''
            <!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
             "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
            <busconfig>
              <policy user="greeter">
                <allow own="nl.luukblankenstijn.ContestGreeterService"/>
              </policy>
              <policy context="default">
                <allow send_destination="nl.luukblankenstijn.ContestGreeterService"/>
              </policy>
            </busconfig>
          '')
        ];
      })
    ]
  );
}
