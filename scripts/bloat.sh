#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."

cargo bloat --release --bin versatiles_glyphs --crates
