#![windows_subsystem = "windows"]

use reqwest::blocking;
use serde_json::Value;
use std::{env, fs::File, io};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SystemParametersInfoW,
};

const PICTURE_DIR: &str = "/Pictures/today_bing.jpg";
const BING_JSON_API: &str = "https://cn.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN";
const HOME_ENV: &str = "USERPROFILE";

fn main() {
    // 构造本地保存路径
    let wallpaper_path = env::var(HOME_ENV).unwrap() + PICTURE_DIR;

    // 请求 Bing 官方 JSON 接口
    let response = blocking::get(BING_JSON_API).unwrap().text().unwrap();

    let json: Value = serde_json::from_str(&response).unwrap();

    // 提取 urlbase 字段
    let urlbase = json["images"][0]["urlbase"].as_str().unwrap();

    // 拼接出 UHD 图片的完整 URL
    let image_url = format!("https://cn.bing.com{}_UHD.jpg", urlbase);

    // 下载图片
    let image_bytes = blocking::get(&image_url).unwrap().bytes().unwrap();

    // 保存到本地
    let mut file = File::create(&wallpaper_path).unwrap();
    io::copy(&mut io::Cursor::new(image_bytes), &mut file).unwrap();

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
