#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all -- --check
cargo test
candidate=$(pwd)
downstream=$(mktemp -d "$RUNNER_TEMP/wasm-redox.XXXXXXXX")
trap 'rm -rf "$downstream"' EXIT
git clone --depth=1 --branch main https://github.com/codeandsolder/js-redox.git "$downstream"
cd "$downstream"
echo "Downstream: $(git rev-parse HEAD)" >> "$GITHUB_STEP_SUMMARY"
sed -i "s|^waffle = .*|waffle = { path = \"$candidate\" }|" Cargo.toml
test "$(sed -n '/^waffle = /p' Cargo.toml)" = "waffle = { path = \"$candidate\" }"
cargo test -p wasm-redox
