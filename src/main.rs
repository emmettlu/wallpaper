#![windows_subsystem = "windows"]
use std::{
    env, fs,
    net::{SocketAddr, TcpStream},
    path::Path,
    thread,
    time::Duration,
};

use chrono::Local;
use reqwest::blocking;
use serde_json::Value;

const TEST_ADDR: ([u8; 4], u16) = ([223, 5, 5, 5], 53);
const PICTURE_DIR: &str = "/Pictures/today_bing.jpg";
const BING_JSON_API: &str = "https://cn.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN";
const HOME_ENV: &str = if cfg!(windows) { "USERPROFILE" } else { "HOME" };

fn main() {
    let wallpaper_path = env::var(HOME_ENV).unwrap() + PICTURE_DIR;

    if should_download(&wallpaper_path) {
        #[cfg(unix)]
        unsafe {
            env::remove_var("HTTP_PROXY");
            env::remove_var("HTTPS_PROXY");
        }

        while TcpStream::connect(SocketAddr::from(TEST_ADDR)).is_err() {
            thread::sleep(Duration::from_secs(2))
        }

        let response = blocking::get(BING_JSON_API).unwrap().text().unwrap();
        let json: Value = serde_json::from_str(&response).unwrap();
        let urlbase = json["images"][0]["urlbase"].as_str().unwrap();
        let image_url = format!("https://cn.bing.com{urlbase}_UHD.jpg");
        let image_bytes = blocking::get(&image_url).unwrap().bytes().unwrap();

        fs::write(&wallpaper_path, &image_bytes).unwrap();

        #[cfg(windows)]
        windows_set_wallpaper(&wallpaper_path)
    } else {
        #[cfg(unix)]
        std::process::exit(1)
    }
}

#[inline(always)]
fn should_download(path: &str) -> bool {
    let path = Path::new(path);
    match fs::metadata(path) {
        Ok(metadata) if metadata.len() > 0 => {
            if let Ok(modified) = metadata.modified() {
                let datetime: chrono::DateTime<Local> = modified.into();
                let now = Local::now();
                datetime.date_naive() != now.date_naive()
            } else {
                true
            }
        }
        _ => true,
    }
}

#[cfg(windows)]
#[inline(always)]
fn windows_set_wallpaper(path: &str) {
    use futures::executor::block_on;
    use windows::{
        Storage::StorageFile,
        System::UserProfile::LockScreen,
        Win32::{
            System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize},
            UI::WindowsAndMessaging::{
                SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SystemParametersInfoW,
            },
        },
        core::HSTRING,
    };

    // Common wallpaper
    let wallpaper_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wallpaper_path.as_ptr() as *mut _),
            SPIF_UPDATEINIFILE,
        )
        .unwrap()
    }

    // Lockscreen wallpaper
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED).unwrap();
    }
    block_on(async {
        let image_file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path))
            .unwrap()
            .await
            .unwrap();
        LockScreen::SetImageFileAsync(&image_file)
            .unwrap()
            .await
            .unwrap()
    });
}
