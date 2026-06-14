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

      postPatch = ''
        sed -i "s|site-root[[:space:]]*=[[:space:]]*\"target/site\"|site-root = \"$out/app/site\"|" Cargo.toml
      '';

      buildPhaseCargoCommand = ''
        export HOME=$(mktemp -d)
        cargo-leptos build --release
      '';

      installPhase = ''
        mkdir -p $out/bin
        mkdir -p $out/app/site

        cp target/release/server $out/bin/chaira

        cp -r $out/app/site/* $out/app/site/ || true

        if [ -d "target/site" ]; then
          cp -r target/site/* $out/app/site/
        fi
      '';
    })
