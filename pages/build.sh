#!/bin/sh
set -e
cd "$(dirname "$0")/.."

cargo build -q --bins --features=cli --release
target/release/versatiles_glyphs merge -o pages/web/assets/glyphs/noto_sans_regular testdata/Noto\ Sans/*.ttf
