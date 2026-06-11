{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.chaira;
in {
  options.services.myCustomService = {
    enable = lib.mkEnableOption "chaira";

    # You can add more options here later (ports, packages, etc.)
    # port = lib.mkOption { ... };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.chaira = {
      description = "Chaira service";
      wantedBy = ["multi-user.target"];
      after = ["network.target"];
      serviceConfig = {
        ExecStart = "${pkgs.hello}/bin/hello";
        Restart = "on-failure";
      };
    };
    # You can also open firewall ports, create users, etc.
    networking.firewall.allowedTCPPorts = [3000];
  };
}
