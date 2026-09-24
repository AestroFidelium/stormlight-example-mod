#!/usr/bin/env bash
# Build the guest and lay it out the way the host loads a mod:
#
#     dist/<id>/manifest.toml
#     dist/<id>/<entry>.wasm
#
# That folder is a complete mod package — point `modload` (or a server's mod
# directory) at it. Zipping the folder's contents gives the `.zip` form.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET=wasm32-unknown-unknown

field() { sed -n "s/^$1 *= *\"\\(.*\\)\" *$/\\1/p" "$ROOT/manifest.toml" | head -1; }
id="$(field id)"
entry="$(field entry)"

(cd "$ROOT" && cargo build --release --target "$TARGET" "$@")

out="$ROOT/dist/$id"
rm -rf "$out"
mkdir -p "$out"
cp "$ROOT/manifest.toml" "$out/"
cp "$ROOT/target/$TARGET/release/$entry" "$out/"
echo "$out ($(stat -c %s "$out/$entry") bytes)"
