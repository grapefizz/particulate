{
  description = "Particulate: Falling Sand WASM (Rust)";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable.latest.default;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rust pkgs.wasm-pack pkgs.binaryen pkgs.nodejs pkgs.python3 ];
          RUSTFLAGS = "-C target-feature=+atomics,+bulk-memory,+mutable-globals";
        };
      }
    );
}
