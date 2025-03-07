#!/usr/bin/env bash

set -e

cargo modules dependencies --lib --all-features --no-externs --no-fns --no-traits --no-types | dot -Tsvg > graph.svg
