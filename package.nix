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
      pkgs.makeWrapper
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
        cargo build --release
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
        mkdir -p $out/target/release
        mkdir -p $out/target/site

        cp -r target/release/server $out/target/release/
        cp -r target/site/* $out/target/site
      '';
    })
