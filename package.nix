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
}:
rustPlatform.buildRustPackage rec {
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
  ];

  buildInputs = [
    openssl
    glib
  ];

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
