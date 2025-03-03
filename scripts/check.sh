#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."

PROJECT_DIR=$(pwd)

cargo fmt -- --check
cargo check --all-features --all-targets
cargo clippy --all-features --all-targets
cargo test --all-features --all-targets
