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

for p in duktape paclib proxy_client proxydetox; do
  cargo clean -p $p
  cargo build -p $p
  cargo test -p $p
done

grcov "${profrawdir}" \
  --binary-path ./target/debug/ \
  -s . \
  -t html \
  -o ./coverage/ \
  --ignore '*/build.rs' \
  --ignore '*/src/main.rs' \
  --ignore '*/tests/*' \
  --ignore '*-sys-*' \
  --ignore 'duktape-sys/*' \
  --ignore-not-existing
