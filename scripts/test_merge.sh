#! /bin/bash

cargo build -q --release --bins
time ./target/release/versatiles_glyphs merge ../versatiles-fonts/fonts/Noto\ Sans/Noto\ Sans\ *Regular.ttf -t > /dev/null
