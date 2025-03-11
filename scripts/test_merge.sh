#!/bin/bash

cargo build -q --bins
time ./target/debug/versatiles_glyphs merge ../versatiles-fonts/fonts/Noto\ Sans/Noto\ Sans\ *Regular.ttf
