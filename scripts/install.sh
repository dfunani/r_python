#!/usr/bin/env bash
# Install rpythonc from GitHub Releases into ~/.local/bin (or PREFIX).
set -euo pipefail

REPO="${RPYTHON_REPO:-dfunani/r_python}"
VERSION="${RPYTHON_VERSION:-}"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${PREFIX}/bin"

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Linux)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-gnu" ;;
        aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
        *) echo "unsupported Linux arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        arm64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *) echo "unsupported macOS arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "unsupported OS: $os (build from source: docs/INSTALL.md)" >&2
      exit 1
      ;;
  esac
}

latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": *"v\?\([^"]*\)".*/\1/p' \
    | head -1
}

main() {
  local target asset url tmp
  target="$(detect_target)"

  if [[ -z "$VERSION" ]]; then
    VERSION="$(latest_version)"
    if [[ -z "$VERSION" ]]; then
      echo "No GitHub release found. Set RPYTHON_VERSION or build from source." >&2
      exit 1
    fi
  fi

  asset="rpythonc-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/v${VERSION}/${asset}"

  echo "Installing rpythonc v${VERSION} for ${target} -> ${BIN_DIR}"
  mkdir -p "$BIN_DIR"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT

  curl -fsSL "$url" -o "${tmp}/${asset}"
  tar -xzf "${tmp}/${asset}" -C "$tmp"
  install -m 755 "${tmp}/rpythonc-${target}/rpythonc" "${BIN_DIR}/rpythonc"

  if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo ""
    echo "Add to your shell profile:"
    echo "  export PATH=\"${BIN_DIR}:\$PATH\""
  fi

  echo ""
  "${BIN_DIR}/rpythonc" --version 2>/dev/null || true
  echo "Done. Try: rpythonc --run examples/hello.rpy"
}

main "$@"
