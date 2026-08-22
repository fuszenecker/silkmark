#!/bin/sh
set -eu

printf 'rustc: '
rustc --version
printf 'cargo: '
cargo --version

cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
