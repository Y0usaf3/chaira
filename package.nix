{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  glib,
  cargo-leptos,
  wasm-bindgen-cli,
  binaryen,
  tailwindcss_4,
  stdenv,
  libclang,
}:
rustPlatform.buildRustPackage {
  pname = "chaira";
  version = "0.0.1";

  src = lib.cleanSource ./.;

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    cargo-leptos
    wasm-bindgen-cli
    binaryen
    tailwindcss_4
    libclang
  ];

  buildInputs = [
    openssl
    glib
    stdenv.cc
  ];

  LIBCLANG_PATH = "${libclang.lib}/lib";
  BINDGEN_EXTRA_CLANG_ARGS = "-I${stdenv.cc.libc.dev}/include";

  buildPhase = ''
    export HOME=$(mktemp -d)
    cargo-leptos build --release
  '';

  installPhase = ''
    mkdir -p $out/bin $out/share/chaira

    cp target/release/chaira $out/bin/

    cp -r target/site $out/share/chaira/site
  '';

  meta = with lib; {
    description = "Chaira Leptos Application";
    license = licenses.mit;
  };
}
