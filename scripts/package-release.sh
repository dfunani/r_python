#!/usr/bin/env bash
# Package rpythonc for local maintainer builds (matches CI release layout).
set -euo pipefail

TARGET="${1:?usage: package-release.sh <target-triple> [version]}"
VERSION="${2:-$(tr -d 'v' < VERSION 2>/dev/null || echo 0.0.0)}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="target/${TARGET}/release/rpythonc"
if [[ ! -f "$BIN" ]]; then
  BIN="target/release/rpythonc"
fi
[[ -f "$BIN" ]] || {
  echo "missing binary — run: cargo build -p rpython_cli --release --target ${TARGET}" >&2
  exit 1
}

mkdir -p dist
cp "$BIN" dist/rpythonc
cp LICENSE README.md INSTALL.md dist/
chmod +x dist/rpythonc

ARCHIVE="rpythonc-${VERSION}-${TARGET}.tar.gz"
tar -czf "$ARCHIVE" -C dist .
shasum -a 256 "$ARCHIVE" > "${ARCHIVE}.sha256"

# Unversioned alias (same as CI latest-download aliases)
cp "$ARCHIVE" "rpythonc-${TARGET}.tar.gz"
cp "${ARCHIVE}.sha256" "rpythonc-${TARGET}.tar.gz.sha256"

echo "Created ${ARCHIVE} and rpythonc-${TARGET}.tar.gz"
