#!/bin/sh
set -e
cd "$(dirname "$0")/.."

cargo build -q --bins --release
target/release/versatiles_glyphs merge -o pages/web/assets/glyphs/ testdata/Noto\ Sans/*.ttf
