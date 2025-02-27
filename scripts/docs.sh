#!/usr/bin/env bash
cd "$(dirname "$0")/.."

rm -rf doc
cargo doc --lib --no-deps
