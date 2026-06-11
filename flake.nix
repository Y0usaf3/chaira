{
  description = "chaira dev shell";

  # ========================================
  # Chaira Development Environment
  # ========================================
  # This flake sets up a complete development environment for the Chaira project.
  #
  # Project Structure:
  #   /                     - Root project directory
  #   src/                  - Rust source code
  #   target/               - Build artifacts and compiled binaries
  #   .dev-logs/            - Development service logs (created at shell startup)
  #     ├── surreal.log     - SurrealDB server logs
  #     └── redis.log       - Redis server logs
  #   charli/               - Character/entity data folder
  #
  # Services Running:
  #   - SurrealDB           - In-memory database (default port: 8000)
  #   - Redis               - Cache/session store (default port: 6379)
  #
  # ========================================

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    surrealdb-bin.url = "github:dmitriiStepanidenko/surrealdb-nixos";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    surrealdb-bin,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    overlays = [(import rust-overlay)];
    pkgs = import nixpkgs {
      config.allowUnfree = true;
      inherit overlays system;
    };
    wasmBindgenBin = pkgs.stdenv.mkDerivation rec {
      pname = "wasm-bindgen";
      version = "0.2.123";

      src = pkgs.fetchurl {
        url = "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${version}/wasm-bindgen-${version}-x86_64-unknown-linux-musl.tar.gz";
        hash = "sha256-gPxcHVwSj9Z+mbFDGO6r9537rfRZ7OLTi6k6guVXMMY=";
      };

      nativeBuildInputs = [pkgs.autoPatchelfHook];

      buildInputs = [pkgs.stdenv.cc.cc.lib];

      installPhase = ''
        mkdir -p $out/bin
        cp wasm-bindgen $out/bin/
        cp wasm-bindgen-test-runner $out/bin/
      '';
    };
  in {
    nixosModules.default = import ./chaira.nix;
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        (rust-bin.stable.latest.default.override
          {
            extensions = ["rust-src" "rust-analyzer" "clippy"];
            targets = ["wasm32-unknown-unknown"];
          })
        openssl
        glib
        bacon
        opencode
        llvmPackages.libclang
        tailwindcss_4
        wasmBindgenBin
        cargo-leptos
        leptosfmt
        binaryen
        stdenv.cc.cc.lib
        surrealist # used for debugging ig
        redis
        surrealdb-bin.packages.${system}.latest
      ];

      RUST_LOG = "info";
      SURREAL_BUCKET_FOLDER_ALLOWLIST = "/purrjects/chara/charli/";
      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.stdenv.cc.libc.dev}/include";

      nativeBuildInputs = with pkgs; [
        pkg-config
        stdenv.cc
      ];

      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

      shellHook =
        ''
          RED='\033[0;31m'
          GREEN='\033[0;32m'
          YELLOW='\033[1;33m'
          BLUE='\033[0;34m'
          MAGENTA='\033[0;35m'
          CYAN='\033[0;36m'
          NC='\033[0m' # No Color

          log_info() {
            echo -e "''${BLUE}[INFO]''${NC} $1"
          }

          log_success() {
            echo -e "''${GREEN}[✓]''${NC} $1"
          }

          log_error() {
            echo -e "''${RED}[✗]''${NC} $1"
          }

          log_warn() {
            echo -e "''${YELLOW}[!]''${NC} $1"
          }

          mkdir -p .dev-logs

          start_surreal() {
            # Kill existing SurrealDB if running
            pkill -f "surreal start" > /dev/null 2>&1
            sleep 0.3

            log_info "Starting SurrealDB..."
            surreal start --allow-scripting --allow-experimental files,surrealism --user test --pass test memory > .dev-logs/surreal.log 2>&1 &
            SURREAL_PID=$!
            sleep 0.5

            if kill -0 $SURREAL_PID 2>/dev/null; then
              log_success "SurrealDB running (PID: $SURREAL_PID)"
              echo $SURREAL_PID > .dev-logs/surreal.pid
            else
              log_error "Failed to start SurrealDB"
              return 1
            fi
          }

          start_redis() {
            pkill -f "redis-server" > /dev/null 2>&1
            sleep 0.3

            log_info "Starting Redis..."
            redis-server --port 6379 --loglevel notice --requirepass test > .dev-logs/redis.log 2>&1 &
            REDIS_PID=$!
            sleep 0.5

            if kill -0 $REDIS_PID 2>/dev/null; then
              log_success "Redis running (PID: $REDIS_PID)"
              echo $REDIS_PID > .dev-logs/redis.pid
            else
              log_error "Failed to start Redis"
              return 1
            fi
          }

          cleanup() {
            log_warn "Shutting down services..."
            pkill -f "surreal start" > /dev/null 2>&1
            pkill -f "redis-server" > /dev/null 2>&1
            log_info "Services stopped"
          }

          trap cleanup EXIT

          # Start both services
          start_surreal
          start_redis

          echo ""
          log_info "Chaira's Dev Environment is ready! UwU"
          echo ""
          echo -e "''${CYAN}Available commands:''${NC}"
          echo -e "  ''${MAGENTA}watch-surreal''${NC}    - Tail SurrealDB logs"
          echo -e "  ''${MAGENTA}watch-redis''${NC}      - Tail Redis logs"
          echo -e "  ''${MAGENTA}restart-surreal''${NC}  - Restart SurrealDB (clears memory data)"
          echo -e "  ''${MAGENTA}restart-redis''${NC}    - Restart Redis"
          echo ""
        ''
        + ''
          alias watch-surreal="tail -f .dev-logs/surreal.log"
          alias watch-redis="tail -f .dev-logs/redis.log"

          restart-surreal() {
            pkill -f "surreal start" > /dev/null 2>&1
            sleep 0.3
            start_surreal
          }

          restart-redis() {
            pkill -f "redis-server" > /dev/null 2>&1
            sleep 0.3
            start_redis
          }
        '';
    };
  };
}
