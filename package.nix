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
    src = craneLib.cleanSource ./.;
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

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;

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
