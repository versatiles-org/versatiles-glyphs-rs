#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

cargo +nightly udeps -q --bins
cargo +nightly udeps -q --lib --no-default-features
