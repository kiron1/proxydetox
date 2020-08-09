#!/bin/bash
set -eu
cd $GITHUB_WORKSPACE
cargo build --release --target=x86_64-unknown-linux-musl
