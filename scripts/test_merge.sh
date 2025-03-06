#! /bin/bash

cargo build -q --release --bins
time ./target/release/versatiles_glyphs merge -o temp ../versatiles-fonts/fonts/Noto\ Sans/Noto\ Sans\ *Regular.ttf
