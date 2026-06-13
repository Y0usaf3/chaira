{
  lib,
  pkgs,
  craneLib,
  pkg-config,
  openssl,
  glib,
  cargo-leptos,
  wasm-bindgen-cli,
  binaryen,
  tailwindcss_4,
  stdenv,
  libclang,
  llvmPackages,
}: let
  commonArgs = {
    pname = "chaira";
    version = "0.0.1";
    strictDeps = true;

    nativeBuildInputs = [
      pkg-config
      cargo-leptos
      wasm-bindgen-cli
      binaryen
      tailwindcss_4
      libclang
      llvmPackages.bintools
    ];

    buildInputs = [
      openssl
      glib
      stdenv.cc
    ];

    LIBCLANG_PATH = "${libclang.lib}/lib";
    BINDGEN_EXTRA_CLANG_ARGS = "-I${stdenv.cc.libc.dev}/include";
  };

  # 1. Cache only dependencies: filters down to just Cargo.toml/lock and .rs files
  cargoArtifacts = craneLib.buildDepsOnly (commonArgs
    // {
      src = craneLib.cleanCargoSource ./.;
    });
in
  craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;

      src = lib.cleanSource ./.;

      buildPhase = ''
        export HOME=$(mktemp -d)
        cargo-leptos build --release
      '';

      installPhase = ''
        mkdir -p $out/bin $out/share/chaira
        cp target/release/server $out/bin/chaira
        cp -r target/site $out/share/chaira/site
      '';
    })
