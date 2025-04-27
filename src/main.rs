#![windows_subsystem = "windows"]

use std::{
    env, fs, io,
    net::TcpStream,
    path::Path,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use reqwest::blocking;
use serde_json::Value;
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SystemParametersInfoW,
};

const PICTURE_DIR: &str = "/Pictures/today_bing.jpg";
const BING_JSON_API: &str = "https://cn.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN";
const HOME_ENV: &str = "USERPROFILE";

fn main() {
    // 构造本地保存路径
    let wallpaper_path = env::var(HOME_ENV).unwrap() + PICTURE_DIR;

    if should_download(&wallpaper_path) {
        // 检查网络连接
        while TcpStream::connect("223.5.5.5:53").is_err() {
            thread::sleep(Duration::from_secs(3))
        }

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
        let mut file = fs::File::create(&wallpaper_path).unwrap();
        io::copy(&mut io::Cursor::new(image_bytes), &mut file).unwrap();
    }

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

#[inline(always)]
fn should_download(path: &str) -> bool {
    let path = Path::new(path);
    if path.exists() {
        let metadata = fs::metadata(path).unwrap();
        if metadata.len() > 0 {
            // 文件不为空
            // 获取文件的最后修改时间
            let modified_time = metadata.modified().unwrap();
            // 获取当前系统时间
            let current_time = SystemTime::now();

            // 将两个时间转换为自 UNIX_EPOCH 以来的秒数
            let modified_secs = modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
            let current_secs = current_time.duration_since(UNIX_EPOCH).unwrap().as_secs();

            // 计算日期（天数）
            let modified_day = modified_secs / (24 * 3600);
            let current_day = current_secs / (24 * 3600);

            // 如果不在同一天，则需要下载
            modified_day != current_day
        } else {
            true // 文件为空，需要下载
        }
    } else {
        true // 文件不存在，需要下载
    }
}
