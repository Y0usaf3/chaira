{
  lib,
  stdenv,
  pkgs,
  craneLib,
  pkg-config,
  openssl,
  glib,
  cargo-leptos,
  wasm-bindgen-cli,
  binaryen,
  tailwindcss_4,
  libclang,
  llvmPackages,
}: let
  rustSrcOnly = craneLib.cleanCargoSource ./.;
  fullAppSrc = lib.cleanSource ./.;

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

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs
    // {
      src = rustSrcOnly;

      buildPhaseCargoCommand = ''
        cargo build --release --bin server --no-default-features --features ssr
      '';
    });
in
  craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;

      src = fullAppSrc;

      doNotPostBuildInstallCargoBinaries = true;

      buildPhaseCargoCommand = ''
        export HOME=$(mktemp -d)
        cargo-leptos build --release
      '';

      installPhase = ''
        mkdir -p $out/app/release/
        mkdir -p $out/app/site/

        cp target/release/server $out/app/release/server
        cp -r target/site $out/app/site
      '';
    })
