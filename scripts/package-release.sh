#!/usr/bin/env bash
# Package rpythonc for CI release (run from repo root after cargo build --release).
set -euo pipefail

TARGET="${1:?usage: package-release.sh <target-triple>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="target/release/rpythonc"
[[ -f "$BIN" ]] || { echo "missing $BIN — run: cargo build -p rpython_cli --release" >&2; exit 1; }

STAGE="dist/rpythonc-${TARGET}"
rm -rf "$STAGE" "dist/rpythonc-${TARGET}.tar.gz"
mkdir -p "$STAGE"
cp "$BIN" "$STAGE/rpythonc"
cp LICENSE README.md "$STAGE/"
chmod +x "$STAGE/rpythonc"

tar -czf "dist/rpythonc-${TARGET}.tar.gz" -C dist "rpythonc-${TARGET}"
(
  cd dist
  shasum -a 256 "rpythonc-${TARGET}.tar.gz" > "rpythonc-${TARGET}.tar.gz.sha256"
)

echo "Created dist/rpythonc-${TARGET}.tar.gz"
