#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."

PROJECT_DIR=$(pwd)

cargo check
cargo fmt -- --check
cargo clippy
cargo test
