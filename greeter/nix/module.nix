flake:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.greetd.loom-greeter;
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
    background_label = cfg.backgroundLabel;
    background_label_color = cfg.backgroundLabelColor;
    session = cfg.session;
    username = cfg.username;
    password = cfg.password;
    url = cfg.url;
  };

  # Filter out null values
  filteredConfig = lib.filterAttrs (_: v: v != null) greeterConfig;

  configFile = tomlFormat.generate "loom-greeter.toml" filteredConfig;

  greeterPackage = flake.packages.${pkgs.system}.loom-greeter;
in
{
  options.services.greetd.loom-greeter = {
    enable = mkEnableOption "loom-greeter, a greetd greeter for icpc contests with the loom system";

    package = mkOption {
      type = types.package;
      default = greeterPackage;
      description = "The loom-greeter package to use.";
    };

    westonPackage = mkOption {
      type = types.package;
      default = pkgs.weston;
      description = "The weston package to use";
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
      type =
        with types;
        either str (submodule {
          options.command = mkOption {
            type = str;
            description = "The command to execute to retrieve the key sequence.";
          };
        });
      default = "chain";
      description = ''
        Key sequence to toggle the login UI.
        Can be a plain string or an attribute set: { command = "get-chain-script"; }.
      '';
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

    backgroundLabel = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Label to display on the background";
      example = "team 1";
    };

    backgroundLabelColor = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Hex code of the color of the background label";
      example = "#ffffff";
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
      type =
        with types;
        either str (submodule {
          options.command = mkOption {
            type = str;
            description = "The command to execute to retrieve the secret.";
          };
        });
      default = "";
      description = ''
        The password for automatic login. 
        Can be a plain string or an attribute set like { command = "my-cmd"; }.
      '';
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
            command = "${cfg.westonPackage}/bin/weston --shell=kiosk-shell.so -- ${pkgs.systemd}/bin/systemd-cat -t loom-greeter ${cfg.package}/bin/loom-greeter ${configFile}";
            user = "greeter";
          };
        };
      }
      # Only add D-Bus if enabled
      (mkIf cfg.enableDbus {
        services.dbus.packages = [
          (pkgs.writeTextDir "share/dbus-1/system.d/nl.luukblankenstijn.loom.GreeterService.conf" ''
            <!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
             "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
            <busconfig>
              <policy user="greeter">
                <allow own="nl.luukblankenstijn.loom.GreeterService"/>
              </policy>
              <policy context="default">
                <allow send_destination="nl.luukblankenstijn.loom.GreeterService"/>
              </policy>
            </busconfig>
          '')
        ];
      })
    ]
  );
}
