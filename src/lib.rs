use std::{fs, path::Path, thread, time::Duration};

use chrono::Local;

pub const RETRY_DELAY: std::time::Duration = Duration::from_secs(2);

#[inline(always)]
pub fn should_download(wallpaper_path: &Path) -> bool {
    fs::metadata(wallpaper_path)
        .ok()
        .filter(|metadata| metadata.len() > 0)
        .and_then(|metadata| metadata.modified().ok())
        .map(|modified| {
            let datetime: chrono::DateTime<Local> = modified.into();
            datetime.date_naive() != Local::now().date_naive()
        })
        .unwrap_or(true)
}

#[inline(always)]
pub fn set_wallpaper(wallpaper_path: &Path) {
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
            use cidre::ns;

            wallpaper_path
                .parent().and_then(|dir| fs::read_dir(dir).ok()).into_iter()
                .flatten().flatten().map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .map(|n| {
                            n.to_string_lossy().starts_with("today_bing_")
                        })
                        .unwrap_or(false)
                })
                .for_each(|path| {
                    let _ = fs::remove_file(path);
                });

            let tmp = wallpaper_path
                .with_file_name(format!("today_bing_{}.jpg", Local::now().format("%Y%m%d")));
            fs::copy(wallpaper_path, &tmp).unwrap();
            let wallpaper_path = tmp;

            let url = ns::Url::with_fs_path_str(wallpaper_path.to_str().unwrap(), false);
            let workspace = ns::Workspace::shared();
            let options = ns::Dictionary::new();
            let mut screens = ns::Screen::screens();

            while screens.is_empty() {
                thread::sleep(RETRY_DELAY);
                screens = ns::Screen::screens()
            }

            for screen in screens.iter() {
                while workspace
                    .set_desktop_image_url(&url, screen, &options)
                    .is_err()
                {
                    thread::sleep(RETRY_DELAY)
                }
            }
        }
    }
}
