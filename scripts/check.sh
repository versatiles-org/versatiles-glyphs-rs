#!/usr/bin/env bash
cd "$(dirname "$0")/.."

echo "cargo fmt"
result=$(CARGO_TERM_COLOR=always cargo fmt -- --check 2>&1)
if [ "$result" != "" ]; then
	echo -e "$result\nERROR DURING: cargo fmt"
	exit 1
fi

echo "cargo check"
result=$(CARGO_TERM_COLOR=always cargo check --workspace --all-features --all-targets 2>&1)
if [ $? -ne 0 ]; then
	echo -e "$result\nERROR DURING: cargo check"
	exit 1
fi

echo "cargo clippy"
result=$(CARGO_TERM_COLOR=always cargo clippy --all-features --all-targets -- -W missing_docs -D warnings 2>&1)
if [ $? -ne 0 ]; then
	echo -e "$result\nERROR DURING: cargo clippy $1"
	exit 1
fi

echo "cargo test"
result=$(CARGO_TERM_COLOR=always cargo test --all-features --all-targets 2>&1)
if [ $? -ne 0 ]; then
	echo -e "$result\nERROR DURING: cargo test $1"
	exit 1
fi
