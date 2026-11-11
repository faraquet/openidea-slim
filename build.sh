#!/bin/sh
set -eu
cd "$(dirname "$0")"
cargo build --locked --release --target wasm32-wasip2 --lib
mkdir -p dist
cp "${CARGO_TARGET_DIR:-target}/wasm32-wasip2/release/openidea_slim.wasm" dist/plugin.wasm
python3 - <<'PY'
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

package = Path("faraquet.slim-0.2.1.oiplugin")
with ZipFile(package, "w", compression=ZIP_DEFLATED) as archive:
    for name in ["plugin.toml", "languages/slim.toml", "dist/plugin.wasm", "icons/slim.svg", "README.md", "LICENSE", "OPENIDEA-LICENSE", "THIRD_PARTY_NOTICES.txt"]:
        archive.write(name, name)
print(f"Created {package} ({package.stat().st_size:,} bytes)")
PY
