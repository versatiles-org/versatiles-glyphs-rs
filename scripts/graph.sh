#!/usr/bin/env bash

set -e

cargo modules dependencies --lib --all-features --no-externs --no-traits --no-types --layout neato \
	| sed -e 's/color="#7f7f7f"/color="#00000033"/g' \
	| dot -Gmode=sgd -Gmaxiter=10000 -Elen=100 -Tsvg \
	> graph.svg

   #| sed -e 's/fontsize="10"/fontsize="5"/g' \
