{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        cargoToml = pkgs.lib.importTOML ./Cargo.toml;
      in
        with pkgs; {
          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = false;
          };

          devShells.default = mkShell {
            buildInputs = [
              (rust-bin.stable.latest.default.override {
                extensions = ["rust-src"];
                targets = ["wasm32-unknown-unknown"];
              })
              rust-analyzer
              wasm-bindgen-cli
              cargo-watch
              miniserve
              nodejs_22

              (writeShellScriptBin "serve" ''
                set -euo pipefail
                cargo run
                miniserve "$OUTPUT" --index index.html --interfaces 127.0.0.1 --port 8000 &
                server_process_id=$!
                trap 'kill "$server_process_id" 2>/dev/null || true; wait "$server_process_id" 2>/dev/null || true' EXIT
                printf '%s\n' 'Serving the generated site at http://blog.localhost:8000/'
                cargo watch --postpone --watch src --watch components --watch posts --watch assets --watch Cargo.toml --watch Cargo.lock --exec run
              '')

              (writeShellScriptBin "ci" ''
                set -euo pipefail
                cargo fmt --all -- --check --color always
                cargo clippy --all-features --workspace -- -D warnings
                cargo run
                cargo test --workspace --all-features
                nix build
              '')
            ];
            SOURCE = "posts";
            OUTPUT = "dist";
            SITE_URL = "https://ksevelyar.rusty-cluster.net";
          };
        }
    );
}
