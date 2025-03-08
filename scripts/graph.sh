#!/usr/bin/env bash

set -e

cargo modules dependencies --lib --all-features --no-externs --no-traits --no-types --no-fns --layout neato \
	| sed -e 's/color="#7f7f7f", style="dashed"/color="#00000020", fontcolor="#00000020"/g' \
	| dot -Gmode=sgd -Gmaxiter=10000 -Elen=100 -Tsvg \
	> graph.svg

   #| sed -e 's/fontsize="10"/fontsize="5"/g' \
