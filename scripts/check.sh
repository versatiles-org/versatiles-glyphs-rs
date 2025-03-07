#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."

PROJECT_DIR=$(pwd)

echo "cargo fmt"
result=$(cargo fmt -- --check 2>&1)
if [ $? -ne 0 ]; then
	echo -e "$result\nERROR DURING: cargo fmt"
	exit 1
fi

echo "cargo check"
result=$(cargo check --workspace --all-features --all-targets 2>&1)
if [ $? -ne 0 ]; then
	echo -e "$result\nERROR DURING: cargo check"
	exit 1
fi

echo "cargo clippy"
result=$(cargo clippy --all-features --all-targets -- -D warnings 2>&1)
if [ $? -ne 0 ]; then
	echo -e "$result\nERROR DURING: cargo clippy $1"
	exit 1
fi

echo "cargo test"
result=$(cargo test --all-features --all-targets 2>&1)
if [ $? -ne 0 ]; then
	echo -e "$result\nERROR DURING: cargo test $1"
	exit 1
fi
