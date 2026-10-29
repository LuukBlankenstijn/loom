flake:
{
  config,
  lib,
  pkgs,
  utils,
  ...
}:

with lib;

let
  cfg = config.services.loom-backend;

  flakePackages = flake.packages.${pkgs.stdenv.hostPlatform.system};

  credentialsDir = "/run/credentials/loom-backend.service";

  port = toInt (last (splitString ":" cfg.listenAddress));

  configFile = (pkgs.formats.toml { }).generate "loom-backend.toml" (
    {
      listen = cfg.listenAddress;
      database_path = "${cfg.dataDir}/loom.db";
      video = {
        multicast_address = cfg.video.multicastAddress;
        multicast_ttl = cfg.video.multicastTtl;
      }
      // optionalAttrs (cfg.video.multicastInterface != null) {
        multicast_interface = cfg.video.multicastInterface;
      };
    }
    // optionalAttrs (cfg.dashboardPackage != null) { dashboard_dir = "${cfg.dashboardPackage}"; }
    // optionalAttrs (cfg.authTokenFile != null) { auth_token_file = "${credentialsDir}/auth-token"; }
    // optionalAttrs cfg.trustedProxyHeader { trusted_proxy_header = true; }
    // optionalAttrs (cfg.contestApi != null) {
      contest_api = {
        base_url = cfg.contestApi.baseUrl;
        inherit (cfg.contestApi) username;
        password_file = "${credentialsDir}/contest-api-password";
      };
    }
  );
in
{
  options.services.loom-backend = {
    enable = mkEnableOption "loom-backend, the loom admin API, station gRPC service and dashboard server";
    package = mkOption {
      type = types.package;
      default = flakePackages.loom-backend;
      description = "The loom-backend package to use.";
    };
    dashboardPackage = mkOption {
      type = types.nullOr types.package;
      default = flakePackages.loom-dashboard;
      description = "The built dashboard served at /; null serves no dashboard.";
    };
    dataDir = mkOption {
      type = types.path;
      default = "/var/lib/loom";
      description = "Directory holding the SQLite database loom.db; only the loom user can read it.";
    };
    listenAddress = mkOption {
      type = types.str;
      default = "0.0.0.0:8080";
      description = "host:port for the dashboard, the HTTP API under /api, gRPC-web under /rpc and the station gRPC service.";
    };
    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Open the listen port; stations need it.";
    };
    authTokenFile = mkOption {
      type = types.nullOr (types.strMatching "/.+");
      default = null;
      example = "/run/agenix/loom-auth-token";
      description = ''
        File holding the admin bearer token. systemd passes it to the service as a credential,
        so it may be readable by root only. Without it the admin API and dashboard are unauthenticated.
        The dashboard never sends the token, so with it the dashboard needs trustedProxyHeader.
      '';
    };
    trustedProxyHeader = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Let gRPC requests carrying an x-proxy-authorized header skip the token check. Only enable this
        when every admin request passes through a reverse proxy that authenticates admins, sets the header
        and strips client-supplied copies; anyone reaching the listen port directly can send it too.
      '';
    };
    contestApi = mkOption {
      type = types.nullOr (
        types.submodule {
          options = {
            baseUrl = mkOption {
              type = types.str;
              example = "https://domjudge.example/domjudge";
              description = "Base URL of the DOMjudge contest API.";
            };
            username = mkOption {
              type = types.str;
              description = "Contest API username.";
            };
            passwordFile = mkOption {
              type = types.strMatching "/.+";
              example = "/run/agenix/loom-contest-api-password";
              description = ''
                File holding the contest API password. systemd passes it to the service as a credential,
                so it may be readable by root only.
              '';
            };
          };
        }
      );
      default = null;
      description = "DOMjudge contest API to sync with; null runs in local mode.";
    };
    video = {
      multicastAddress = mkOption {
        type = types.str;
        default = "239.255.76.1:5004";
        description = ''
          group:port the current YouTube wallpaper is multicast to as MPEG-TS. Stations must accept
          UDP on this port (services.greetd.loom-greeter.openFirewall with a matching videoStreamPort).
        '';
      };
      multicastTtl = mkOption {
        type = types.ints.u8;
        default = 1;
        description = "TTL of the multicast packets; 1 keeps the stream on the local network segment.";
      };
      multicastInterface = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "10.0.0.1";
        description = "Local IPv4 address of the interface to send the stream from; null lets the routing table decide.";
      };
      ytDlpPackage = mkPackageOption pkgs "yt-dlp" { };
      ffmpegPackage = mkPackageOption pkgs "ffmpeg-headless" { };
    };
  };

  config = mkIf cfg.enable {
    users.users.loom = {
      isSystemUser = true;
      group = "loom";
      home = cfg.dataDir;
    };
    users.groups.loom = { };

    systemd.services.loom-backend = {
      description = "Loom backend";
      wants = [ "network-online.target" ];
      after = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      path = [
        cfg.video.ytDlpPackage
        cfg.video.ffmpegPackage
      ];
      serviceConfig = {
        ExecStart = utils.escapeSystemdExecArgs [
          (getExe cfg.package)
          configFile
        ];
        WorkingDirectory = cfg.dataDir;
        StateDirectory = mkIf (hasPrefix "/var/lib/" cfg.dataDir) (removePrefix "/var/lib/" cfg.dataDir);
        StateDirectoryMode = "0700";
        User = "loom";
        Group = "loom";
        UMask = "0077";
        LoadCredential =
          optional (cfg.authTokenFile != null) "auth-token:${cfg.authTokenFile}"
          ++ optional (cfg.contestApi != null) "contest-api-password:${cfg.contestApi.passwordFile}";
        Restart = "always";
        RestartSec = "5s";

        CapabilityBoundingSet = if port < 1024 then [ "CAP_NET_BIND_SERVICE" ] else [ "" ];
        AmbientCapabilities = optional (port < 1024) "CAP_NET_BIND_SERVICE";
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ReadWritePaths = [ cfg.dataDir ];
        PrivateTmp = true;
        PrivateDevices = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        ProtectClock = true;
        ProtectHostname = true;
        ProtectProc = "invisible";
        RestrictAddressFamilies = [
          "AF_INET"
          "AF_INET6"
          "AF_UNIX"
          "AF_NETLINK"
        ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = false;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
          "~@privileged"
        ];
      };
    };

    networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [ port ];

    systemd.tmpfiles.rules = [ "d ${cfg.dataDir} 0700 loom loom -" ];
  };
}
