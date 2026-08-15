#![windows_subsystem = "windows"]

use std::{
    env, fs,
    io::Write,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use chrono::Local;
use ntex::{
    SharedCfg,
    client::ClientBuilder,
    rt::{DefaultRuntime, System},
};
use serde_json::Value;

const BING_JSON_API: &str = "https://cn.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN";
const HOME_ENV: &str = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
const TEST_ADDR: ([u8; 4], u16) = ([223, 5, 5, 5], 443);

fn main() {
    let wallpaper_path = PathBuf::from(env::var(HOME_ENV).unwrap())
        .join("Pictures")
        .join("today_bing.jpg");

    if should_download(&wallpaper_path) {
        unsafe {
            env::remove_var("ALL_PROXY");
            env::remove_var("HTTP_PROXY");
            env::remove_var("HTTPS_PROXY");
        }

        while TcpStream::connect(SocketAddr::from(TEST_ADDR)).is_err() {
            thread::sleep(Duration::from_secs(2))
        }

        let image_bytes = System::new("", DefaultRuntime).block_on(async {
            let client = ClientBuilder::new()
                .disable_timeout()
                .response_payload_limit(usize::MAX)
                .build(SharedCfg::default())
                .await
                .unwrap();

            let response = client.get(BING_JSON_API).send().await.unwrap();
            let body = response.body().await.unwrap();
            let response = String::from_utf8(body.to_vec()).unwrap();
            let json: Value = serde_json::from_str(&response).unwrap();
            let urlbase = json["images"][0]["urlbase"].as_str().unwrap();
            let image_url = format!("https://cn.bing.com{urlbase}_UHD.jpg");
            let img_res = client.get(image_url).send().await.unwrap();
            img_res.body().await.unwrap()
        });

        #[cfg(target_os = "macos")]
        unsafe {
            let mut file = fs::File::create(&wallpaper_path).unwrap_unchecked();
            file.write_all(&image_bytes).unwrap_unchecked();
            file.sync_all().unwrap_unchecked();
        }
    } else {
        #[cfg(target_os = "linux")]
        std::process::exit(1);
    }

    set_wallpaper(&wallpaper_path)
}

#[inline(always)]
fn should_download(wallpaper_path: &Path) -> bool {
    match fs::metadata(wallpaper_path) {
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

#[inline(always)]
fn set_wallpaper(wallpaper_path: &Path) {
    cfg_select! {
        windows => {
            use std::os::windows::ffi::OsStrExt;

            use futures::executor::block_on;
            use windows::{
                Storage::StorageFile,
                System::UserProfile::LockScreen,
                Win32::{
                    System::WinRT::{RO_INIT_SINGLETHREADED, RoInitialize},
                    UI::WindowsAndMessaging::{
                        SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SystemParametersInfoW,
                    },
                },
                core::HSTRING,
            };

            // Desktop wallpaper
            unsafe {
                let wallpaper_path = wallpaper_path
                    .as_os_str()
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect::<Vec<_>>();
                SystemParametersInfoW(
                    SPI_SETDESKWALLPAPER,
                    0,
                    Some(wallpaper_path.as_ptr() as *mut _),
                    SPIF_UPDATEINIFILE,
                )
                .unwrap();
            }

            // Lockscreen wallpaper
            unsafe {
                RoInitialize(RO_INIT_SINGLETHREADED).unwrap();
            }
            block_on(async {
                let wallpaper_path =
                    StorageFile::GetFileFromPathAsync(&HSTRING::from(wallpaper_path))
                        .unwrap()
                        .await
                        .unwrap();
                LockScreen::SetImageFileAsync(&wallpaper_path)
                    .unwrap()
                    .await
                    .unwrap();
            });
        }
        target_os = "macos" => {
            use objc2::MainThreadMarker;
            use objc2_app_kit::{NSScreen, NSWorkspace};
            use objc2_foundation::{NSDictionary, NSString, NSURL};

            let mtm = MainThreadMarker::new().expect("must be called from the main thread");
            let url =
                NSURL::fileURLWithPath(&NSString::from_str(wallpaper_path.to_str().unwrap()));
            let workspace = NSWorkspace::sharedWorkspace();
            let options = NSDictionary::new();

            for screen in NSScreen::screens(mtm) {
                unsafe {
                    workspace
                        .setDesktopImageURL_forScreen_options_error(&url, &screen, &options)
                        .unwrap_unchecked()
                }
            }
        }
    }
}
