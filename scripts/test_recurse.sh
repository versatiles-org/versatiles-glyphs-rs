#! /bin/bash

cargo build -q --bins
# ./target/debug/versatiles_glyphs recurse -t ../versatiles-fonts/fonts/ | gzip -9 > fonts.tar.gz
./target/debug/versatiles_glyphs recurse -t ../versatiles-fonts/fonts/ | gzip -9 > fonts.tar.gz

# cargo build -q --bins --release
# time ./target/release/versatiles_glyphs convert -o temp ../versatiles-fonts/fonts/Noto\ Sans/Noto\ Sans\ *Regular.ttf
