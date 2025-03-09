#! /bin/bash

cargo build -q --bins

time cargo flamegraph --root --dev --bin=versatiles_glyphs -- \
  recurse testdata/Fira* --tar --single-thread >/dev/null
