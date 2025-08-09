#!/usr/bin/env bash
set -e

# Install Rust and wasm-pack if not present
if ! command -v rustc >/dev/null; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  source $HOME/.cargo/env
fi
if ! command -v wasm-pack >/dev/null; then
  cargo install wasm-pack
fi

# Build WASM to public/pkg
wasm-pack build --target web --out-dir ./public/pkg
