#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

mkdir -p bindings/demo-token/src bindings/demo-vault/src

stellar contract bindings typescript \
  --wasm target/wasm32v1-none/release/demo_token.wasm \
  --output-dir "$tmpdir/demo-token"

cp "$tmpdir/demo-token/src/index.ts" bindings/demo-token/src/index.ts

stellar contract bindings typescript \
  --wasm target/wasm32v1-none/release/demo_vault.wasm \
  --output-dir "$tmpdir/demo-vault"

cp "$tmpdir/demo-vault/src/index.ts" bindings/demo-vault/src/index.ts
