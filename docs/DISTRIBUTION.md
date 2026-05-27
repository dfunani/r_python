# Distribution and releases

How maintainers build and ship `rpythonc` (mirrors [r_tvui](https://github.com/dfunani/r_tvui) release layout).

## CI release workflow

On every tag `v*` (e.g. `v1.0.0`), [`.github/workflows/release.yml`](../.github/workflows/release.yml):

1. Builds `rpythonc` for each target with `cargo build -p rpython_cli --release --target <triple>`
2. Packages `rpythonc-<version>-<target>.tar.gz` + SHA256
3. Uploads **stable aliases** `rpythonc-<target>.tar.gz` for `releases/latest/download/...`
4. Creates a GitHub Release with all assets attached

### Matrix targets

| Asset | Platform |
|-------|----------|
| `rpythonc-*-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon |
| `rpythonc-*-x86_64-apple-darwin.tar.gz` | macOS Intel |
| `rpythonc-*-x86_64-unknown-linux-gnu.tar.gz` | Linux x64 |
| `rpythonc-*-aarch64-unknown-linux-gnu.tar.gz` | Linux ARM64 |

## Publish v1.0.0 (maintainer commands)

```bash
cd /path/to/r_python

# Ensure logged in as dfunani, workflow scope for Actions
gh auth status
gh auth refresh -h github.com -s workflow

git add -A
git commit -m "Prepare v1.0.0 release"
git push origin main

git tag v1.0.0
git push origin v1.0.0
```

Or trigger manually without a local tag:

```bash
gh workflow run release.yml -f version=1.0.0
```

Watch the run:

```bash
gh run list --workflow=release.yml
gh run watch
```

Release page: https://github.com/dfunani/r_python/releases

## Local package (same layout as CI)

```bash
rustup target add aarch64-apple-darwin   # example
cargo build -p rpython_cli --release --target aarch64-apple-darwin
./scripts/package-release.sh aarch64-apple-darwin 1.0.0
```

## User install URLs

```bash
# Latest (stable alias)
curl -fL -O https://github.com/dfunani/r_python/releases/latest/download/rpythonc-aarch64-apple-darwin.tar.gz
tar -xzf rpythonc-aarch64-apple-darwin.tar.gz
install -m 755 rpythonc ~/.local/bin/rpythonc

# Or install script
curl -fsSL https://raw.githubusercontent.com/dfunani/r_python/main/scripts/install.sh | bash
```

Versioned filename example: `rpythonc-1.0.0-x86_64-unknown-linux-gnu.tar.gz`
