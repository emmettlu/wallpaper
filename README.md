# wallpaper
Small utility that keeps the Windows desktop and lock screen updated with the daily Bing wallpaper. It fetches the latest image from `https://cn.bing.com` and writes it to `~/Pictures/today_bing.jpg` before applying it at boot time.

## Features
- blocks until an Internet connection is available before attempting to download
- writes the image once per calendar day (skips download if the previous file is still from today)
- updates both the desktop wallpaper and Windows lock screen via native Win32/WinRT APIs

## Requirements
- Rust 2024 toolchain (`rustc` and `cargo`)
- Windows 10+ (the build links against Win32 and WinRT APIs; Unix builds will exit if they cannot set a wallpaper)
- Network access to `https://cn.bing.com` to download the JSON feed and image

## Build
```sh
cargo build --release
```

## Run
```sh
cargo run --release
```
The executable saves the image at `~/Pictures/today_bing.jpg` and only downloads again when the existing file’s modified date differs from the current date.

## Notes
- `chrono`, `reqwest`, `serde_json`, and the Windows-specific crates are included in `Cargo.toml`.
- This project defaults to a release build (optimized, link-time optimized, and stripped). Use `cargo run` if you want faster iteration.
