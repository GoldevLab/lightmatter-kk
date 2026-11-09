#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-x86_64-pc-windows-gnu}"
DIST_DIR="${ROOT}/dist/lightmatter-portable"
ARCHIVE_NAME="${2:-lightmatter-portable.zip}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Falta Cargo / Rust. Instálalo antes de generar la versión portable." >&2
  exit 1
fi

if [[ "$TARGET" == "x86_64-pc-windows-gnu" ]] && ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
  echo "Falta mingw-w64. En Ubuntu/Debian puedes probar: sudo apt-get install mingw-w64" >&2
  exit 1
fi

rustup target add "$TARGET" >/dev/null 2>&1 || true
cargo build --release --target "$TARGET"

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/public" "$DIST_DIR/src"

cp "$ROOT/target/$TARGET/release/lightmatter.exe" "$DIST_DIR/"
cp -R "$ROOT/public/." "$DIST_DIR/public/"
cp -R "$ROOT/src/pages" "$DIST_DIR/src/pages"

mkdir -p "$DIST_DIR/data"
cat > "$DIST_DIR/start-lightmatter.bat" <<'EOF'
@echo off
cd /d "%~dp0"
set RESUMA_ADDR=127.0.0.1:4321
set RESUMA_DATA_DIR=%~dp0data
start "" http://127.0.0.1:4321
lightmatter.exe
EOF

(cd "$ROOT/dist" && zip -r "$ARCHIVE_NAME" "lightmatter-portable" >/dev/null)
echo "Paquete creado: $ROOT/dist/$ARCHIVE_NAME"
