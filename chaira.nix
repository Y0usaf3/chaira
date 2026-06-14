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
        default = "memory";
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
  };

  config = lib.mkIf cfg.enable {
    systemd.services.chaira-surrealdb = {
      description = "Chaira Dev - SurrealDB Instance";
      after = ["network.target"];
      serviceConfig = {
        ExecStart = "${cfg.db.package}/bin/surreal start --allow-scripting --allow-experimental files,surrealism --bind ${cfg.db.host}:${toString cfg.db.port} --user ${cfg.db.user} --pass ${cfg.db.password} ${cfg.db.storage}";
        Restart = "always";
      };
    };

    systemd.services.chaira-redis = {
      description = "Chaira Dev - Redis Instance";
      after = ["network.target"];
      serviceConfig = {
        ExecStart = "${pkgs.redis}/bin/redis-server --bind ${cfg.redis.host} --port ${toString cfg.redis.port} --loglevel notice --requirepass ${cfg.redis.password}";
        Restart = "always";
      };
    };

    systemd.services.chaira = {
      description = "Chaira Leptos Dev Server";
      after = ["network.target" "chaira-surrealdb.service" "chaira-redis.service"];
      requires = ["chaira-surrealdb.service" "chaira-redis.service"];
      wantedBy = ["multi-user.target"];

      environment = {
        B_URL = "${cfg.db.host}:${toString cfg.db.port}";
        DB_USERNAME = cfg.db.user;
        DB_PASSWORD = cfg.db.password;
        MASTER_KEY = cfg.masterKey;
        REDIS_URL = "${cfg.redis.host}:${toString cfg.redis.port}";
        REDIS_PWD = cfg.redis.password;
        RUST_LOG = "info";
        SURREAL_BUCKET_FOLDER_ALLOWLIST = "${toString cfg.src}/charli/";
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.stdenv.cc.libc.dev}/include";
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        LEPTOS_SITE_ROOT = "${chairaPkg}";
      };

      serviceConfig = {
        Type = "simple";

        ExecStart = "${chairaPkg}/bin/chaira";

        WorkingDirectory = "${chairaPkg}";
        Restart = "on-failure";
      };
    };
  };
}
