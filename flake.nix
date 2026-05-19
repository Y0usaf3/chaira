{
  description = "chaira dev shell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    surrealdb-bin.url = "github:dmitriiStepanidenko/surrealdb-nixos";
  };

  outputs = {
    self,
    nixpkgs,
    surrealdb-bin,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      system = "x86_64-linux";
      config.allowUnfree = true;
    };
    # a way to write custom scripts, pretty useful
    # my-custom-script = pkgs.writeShellScriptBin "start-app" ''
    #   echo "Starting backend and frontend..."
    #   cargo run
    # '';
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        cargo
        rustc
        ngrok
        openssl
        glib
        rust-analyzer
        surrealdb-bin.packages.${system}.latest
      ];

      RUST_LOG = "info";
      SURREAL_BUCKET_FOLDER_ALLOWLIST = "./charli/";

      nativeBuildInputs = [pkgs.pkg-config];

      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

      shellHook = ''
        echo "hai in chaira's DEV SHELL UwU"

        start_db() {
          surreal start --allow-experimental files,surrealism --user root --pass root memory > db.log 2>&1 &
          SURREAL_PID=$!
          echo "the database is running ! (PID: $SURREAL_PID)!"
        }

        stop_db() {
          echo "Shutting down SurrealDB..."
          pkill -f "surreal start" > /dev/null 2>&1
        }

        start_db

        trap stop_db EXIT

        alias watch-db="tail -f db.log"

        restart-db() {
          stop_db
          sleep 0.5  # Give the port a split second to clear out
          start_db
        }

        echo "'restart-db' to restart the current db, do it when you want to wipe all the data as the db stores it in memory"
        echo "'watch-db' check the logs whenever you think the db got something to tell you"
      '';
    };
  };
}
