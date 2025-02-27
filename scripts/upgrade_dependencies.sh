#!/usr/bin/env bash
cd "$(dirname "$0")/.."
set -e

echo "Update Rust"
rustup update
rm Cargo.lock

echo "Upgrade Dependencies"
cargo upgrade --incompatible
cargo check --workspace
