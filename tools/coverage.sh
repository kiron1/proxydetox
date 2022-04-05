#!/bin/sh

set -eu

# https://marco-c.github.io/2020/11/24/rust-source-based-code-coverage.html
# https://blog.rust-lang.org/inside-rust/2020/11/12/source-based-code-coverage.html

# cargo install grcov
# rustup component add llvm-tools-preview

profrawdir=$(mktemp -dt proxydetox-profraw.XXXXXX)
trap 'rm -rf ${profrawdir}' EXIT

export RUSTFLAGS="-Zinstrument-coverage"
export LLVM_PROFILE_FILE="${profrawdir}/proxydetox-%p-%m.profraw"
# https://fasterthanli.me/articles/why-is-my-rust-build-so-slow
export RUSTC_BOOTSTRAP=1

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
  --ignore '*/tests/*' \
  --ignore '*-sys-*' \
  --ignore 'duktape-sys/*' \
  --ignore-not-existing
