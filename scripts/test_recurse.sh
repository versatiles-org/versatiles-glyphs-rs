#! /bin/bash

cargo build -q --bins --features=cli
./target/debug/versatiles_glyphs recurse ../versatiles-fonts/fonts/

# cargo build -q --bins --features=cli --release
# time ./target/release/versatiles_glyphs convert -o temp ../versatiles-fonts/fonts/Noto\ Sans/Noto\ Sans\ *Regular.ttf
