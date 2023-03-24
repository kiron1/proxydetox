#!/bin/sh

set -eu

# Requires rust 1.60.0 or newer.

# https://marco-c.github.io/2020/11/24/rust-source-based-code-coverage.html
# https://blog.rust-lang.org/inside-rust/2020/11/12/source-based-code-coverage.html
# https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html

# cargo install grcov
# rustup component add llvm-tools-preview

profrawdir=$(mktemp -dt proxydetox-profraw.XXXXXX)
trap 'rm -rf ${profrawdir}' EXIT

export LLVM_PROFILE_FILE="${profrawdir}/proxydetox-%p-%m.profraw"
export RUSTFLAGS="-C instrument-coverage"

cargo clean
cargo build
cargo test

grcov "${profrawdir}" \
  --binary-path ./target/debug/ \
  -s . \
  -t html \
  -o ./coverage/ \
  --ignore '*/build.rs' \
  --ignore '*/src/main.rs' \
  --ignore '*/tests/*' \
  --ignore-not-existing
