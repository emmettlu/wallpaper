#!/usr/bin/env bash
set -euo pipefail

repository_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app_path="${repository_dir}/target/release/Wallpaper.app"
contents_path="${app_path}/Contents"

if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "The macOS app bundle must be built on macOS." >&2
    exit 1
fi

cd "${repository_dir}"
cargo build --release

rm -rf "${app_path}"
mkdir -p "${contents_path}/MacOS"
cp "packaging/macos/Info.plist" "${contents_path}/Info.plist"
cp "target/release/wallpaper" "${contents_path}/MacOS/wallpaper"
chmod 755 "${contents_path}/MacOS/wallpaper"
codesign --force --sign - "${app_path}"

echo "Created ${app_path}"
