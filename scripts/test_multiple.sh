#! /bin/bash

# cargo build
# ./target/debug/versatiles_glyphs convert -o temp ../versatiles-fonts/fonts/Noto\ Sans/Noto\ Sans\ *Regular.ttf

cargo build -q --release
time ./target/release/versatiles_glyphs convert -o temp ../versatiles-fonts/fonts/Noto\ Sans/Noto\ Sans\ *Regular.ttf
