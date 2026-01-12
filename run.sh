#!/bin/sh

set -e # Exit early if any commands fail

(
  cd "$(dirname "$0")" # Run commands from the script's directory
  cargo build --release --target-dir=/tmp/rust-shell --manifest-path Cargo.toml
)

exec /tmp/rust-shell/release/rust-shell "$@"
