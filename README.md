# wallpaper
Small utility that keeps the Windows and macOS desktop updated with the daily Bing wallpaper. It fetches the latest image from `https://cn.bing.com` and applies it at login.

## Features
- blocks until an Internet connection is available before attempting to download
- writes the image once per calendar day (skips download if the previous file is still from today)
- updates the macOS desktop through AppKit
- updates both the Windows desktop and lock screen via native Win32/WinRT APIs

## Requirements
- Rust 2024 toolchain (`rustc` and `cargo`)
- Windows 10+ or macOS
- Network access to `https://cn.bing.com` to download the JSON feed and image

## Build
```sh
cargo build --release
```

## Run
```sh
cargo run --release
```
The executable saves the image at `~/Pictures/today_bing.jpg` and only downloads once per day.

## macOS app

Build the background app bundle with:

```sh
./scripts/package-macos.sh
```

Move `target/release/Wallpaper.app` to `/Applications`, open it once, then add it in **System Settings > General > Login Items**. The app is an agent app, so it does not open Terminal or show a Dock icon when launched at login.

## Notes
- `chrono`, `ntex`, `serde_json`, and the platform-specific crates are included in `Cargo.toml`.
- This project defaults to a release build (optimized, link-time optimized, and stripped). Use `cargo run` if you want faster iteration.
