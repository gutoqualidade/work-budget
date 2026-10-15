#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
cargo fmt --all -- --check
cargo clippy --all-targets --offline --locked -- -D warnings
cargo test --all-targets --offline --locked
cargo test --doc --offline --locked
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --offline --locked
demo_output=$(mktemp)
trap 'rm -f -- "$demo_output"' EXIT
cargo run --quiet --offline --locked --example lifecycle > "$demo_output"
diff -u docs/demo-output.txt "$demo_output"
printf '%s\n' 'All local checks passed.'
