#!/bin/bash

cargo build -q --bins
./target/debug/versatiles_glyphs debug output/noto_sans_regular -f tsv > new.tsv
./target/debug/versatiles_glyphs debug "temp/basemaps-assets/Noto Sans Regular" -f tsv > old.tsv