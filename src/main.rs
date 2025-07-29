#![windows_subsystem = "windows"]
use std::{
    env, fs, io,
    net::{SocketAddr, TcpStream},
    path::Path,
    thread,
    time::Duration,
};

use chrono::Local;
use reqwest::blocking;
use serde_json::Value;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SystemParametersInfoW,
};

const TEST_ADDR: ([u8; 4], u16) = ([223, 5, 5, 5], 53);
const PICTURE_DIR: &str = "/Pictures/today_bing.jpg";
const BING_JSON_API: &str = "https://cn.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN";
const HOME_ENV: &str = "USERPROFILE";

fn main() {
    let wallpaper_path = env::var(HOME_ENV).unwrap() + PICTURE_DIR;

    if should_download(&wallpaper_path) {
        while TcpStream::connect(SocketAddr::from(TEST_ADDR)).is_err() {
            thread::sleep(Duration::from_secs(3))
        }

        let response = blocking::get(BING_JSON_API).unwrap().text().unwrap();

        let json: Value = serde_json::from_str(&response).unwrap();

        let urlbase = json["images"][0]["urlbase"].as_str().unwrap();

        let image_url = format!("https://cn.bing.com{urlbase}_UHD.jpg");

        let image_bytes = blocking::get(&image_url).unwrap().bytes().unwrap();

        let mut file = fs::File::create(&wallpaper_path).unwrap();
        io::copy(&mut io::Cursor::new(image_bytes), &mut file).unwrap();

        #[cfg(windows)]
        set_windows_wallpaper()
    }
}

#[inline(always)]
fn should_download(path: &str) -> bool {
    let path = Path::new(path);
    if path.exists() {
        let metadata = fs::metadata(path).unwrap();
        if metadata.len() > 0 {
            let modified_time = metadata.modified().unwrap();
            let datetime: chrono::DateTime<chrono::Local> = modified_time.into();
            let now = Local::now();

            datetime.date_naive() != now.date_naive()
        } else {
            true
        }
    } else {
        true
    }
}

#[cfg(windows)]
#[inline(always)]
fn set_windows_wallpaper() {
    let wallpaper_path: Vec<u16> = wallpaper_path
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wallpaper_path.as_ptr() as *mut _),
            SPIF_UPDATEINIFILE,
        )
        .unwrap()
    }
}
