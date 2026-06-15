self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.chaira;
  SurrealDbBin = self.packages.${pkgs.system}.surrealdb;
  chairaPkg = self.packages.${pkgs.system}.chaira;
in {
  options.services.chaira = {
    enable = lib.mkEnableOption "Chaira Live Dev Service";

    src = lib.mkOption {
      type = lib.types.path;
      default = ./.;
      example = "/purrjects/chara";
      description = "Path to your local Chaira repository.";
    };

    db = {
      package = lib.mkOption {
        type = lib.types.package;
        default = SurrealDbBin;
      };
      host = lib.mkOption {
        type = lib.types.str;
        default = "127.0.0.1";
      };
      port = lib.mkOption {
        type = lib.types.port;
        default = 8000;
      };
      user = lib.mkOption {
        type = lib.types.str;
        default = "test";
      };
      password = lib.mkOption {
        type = lib.types.str;
        default = "test";
      };
      storage = lib.mkOption {
        type = lib.types.str;
        default = "rocksdb://proto.db";
      };
    };

    redis = {
      host = lib.mkOption {
        type = lib.types.str;
        default = "127.0.0.1";
      };
      port = lib.mkOption {
        type = lib.types.port;
        default = 6379;
      };
      password = lib.mkOption {
        type = lib.types.str;
        default = "test";
      };
    };

    masterKey = lib.mkOption {
      type = lib.types.str;
      default = "c60fae6c84d0d037c331076747c782a18804c433088a5c72a58180e3b3b32f09";
    };

    hackclub-auth = {
      client_id = lib.mkOption {type = lib.types.str;};
      client_secret = lib.mkOption {type = lib.types.str;};
      redirect_uri = lib.mkOption {type = lib.types.str;};
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.chaira-surrealdb = {
      description = "Chaira Dev - SurrealDB Instance";
      after = ["network.target"];
      wantedBy = ["multi-user.target"];

      serviceConfig = {
        ExecStart = toString [
          "${cfg.db.package}/bin/surreal"
          "start"
          "--allow-scripting"
          "--allow-experimental"
          "files,surrealism"
          "--bind"
          "${cfg.db.host}:${toString cfg.db.port}"
          "--user"
          cfg.db.user
          "--pass"
          cfg.db.password
          cfg.db.storage
        ];
        Restart = "always";

        DynamicUser = true;
        StateDirectory = "chaira-surrealdb";
        WorkingDirectory = "/var/lib/chaira-surrealdb";
      };
    };

    # --- REDIS SERVICE ---
    systemd.services.chaira-redis = {
      description = "Chaira Dev - Redis Instance";
      after = ["network.target"];
      wantedBy = ["multi-user.target"];
      serviceConfig = {
        ExecStart = toString [
          "${pkgs.redis}/bin/redis-server"
          "--bind"
          cfg.redis.host
          "--port"
          (toString cfg.redis.port)
          "--loglevel"
          "notice"
          "--requirepass"
          cfg.redis.password
        ];
        Restart = "always";
        DynamicUser = true;
      };
    };

    systemd.services.chaira = {
      description = "Chaira Leptos Dev Server";
      after = ["network.target" "chaira-surrealdb.service" "chaira-redis.service"];
      requires = ["chaira-surrealdb.service" "chaira-redis.service"];
      wantedBy = ["multi-user.target"];

      environment = {
        DB_URL = "http://${cfg.db.host}:${toString cfg.db.port}";
        DB_USERNAME = cfg.db.user;
        DB_PASSWORD = cfg.db.password;
        MASTER_KEY = cfg.masterKey;
        REDIS_URL = "redis://:${cfg.redis.password}@${cfg.redis.host}:${toString cfg.redis.port}";
        RUST_LOG = "info";
        LEPTOS_SITE_ROOT = "${chairaPkg}";
      };

      serviceConfig = {
        Type = "simple";
        ExecStart = "${chairaPkg}/target/release/server";
        Restart = "on-failure";
      };
    };
  };
}
