{
  description = "chaira dev shell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    surrealdb-bin.url = "github:dmitriiStepanidenko/surrealdb-nixos";
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
    rust-overlay,
    surrealdb-bin,
  }: let
    overlays = [(import rust-overlay)];
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      config.allowUnfree = true;
      inherit overlays system;
    };
    naerskLib = pkgs.callPackage naersk {};
  in {
    packages.${system}.default = naerskLib.buildPackage {
      src = ./.;
      buildInputs = [pkgs.glib];
      nativeBuildInputs = [pkgs.pkg-config];
    };
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        (rust-bin.stable.latest.default.override
          {
            extensions = ["rust-src" "rust-analyzer"];
            targets = [
              "wasm32-wasip1"
              "wasm32-unknown-unknown"
              "x86_64-unknown-linux-musl"
            ];
          })
        ngrok
        openssl
        glib
        rust-analyzer
        wasm-tools
        wasm-pack
        binaryen
        surrealdb-bin.packages.${system}.latest
      ];

      RUST_LOG = "info";
      SURREAL_BUCKET_FOLDER_ALLOWLIST = "/purrjects/chara/charli/";

      nativeBuildInputs = [pkgs.pkg-config];

      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

      shellHook = ''
        echo "hai in chaira's DEV SHELL UwU"

        start_db() {
          surreal start --allow-scripting --allow-experimental files,surrealism --user test --pass test memory > db.log 2>&1 &
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
          sleep 0.5
          start_db
        }

        echo "'restart-db' to restart the current db, do it when you want to wipe all the data as the db stores it in memory"
        echo "'watch-db' check the logs whenever you think the db got something to tell you"
      '';
    };
  };
}
