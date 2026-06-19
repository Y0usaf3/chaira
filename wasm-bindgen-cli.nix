{
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  openssl,
  lib,
  stdenv,
}:
rustPlatform.buildRustPackage rec {
  pname = "wasm-bindgen-cli";
  version = "0.2.125";

  src = fetchFromGitHub {
    owner = "rustwasm";
    repo = "wasm-bindgen";
    rev = version;
    hash = "sha256-7m/QF89PFEoqo8iRr4tPQ5y8v6BM+3O4n4A/8uSnr4k=";
  };

  cargoHash = "sha256-wQh4dP+1B9q9YhqCM/Lhmq1gTPhI05Hhf1v02TCF9Yg=";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl ];

  OPENSSL_NO_VENDOR = 1;

  buildFeatures = [ "clap" ];

  meta = {
    description = "Facilitating high-level interactions between wasm modules and JavaScript";
    homepage = "https://wasm-bindgen.github.io/wasm-bindgen/";
    license = with lib.licenses; [ asl20 mit ];
    mainProgram = "wasm-bindgen";
  };
}
