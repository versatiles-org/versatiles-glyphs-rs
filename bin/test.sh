#! /bin/bash

cargo build
./target/debug/sdf_glyphs convert testdata/Fira\ Sans\ -\ Regular.ttf temp
