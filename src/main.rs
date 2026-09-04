#![windows_subsystem = "windows"]

use std::{
    env, fs,
    io::Write,
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    thread,
};

use wallpaper::{RETRY_DELAY, set_wallpaper, should_download};

use ntex::{
    client::{ClientBuilder, ClientConfig},
    rt::{DefaultRuntime, System},
};
use sonic_rs::{JsonValueTrait, pointer};

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
            thread::sleep(RETRY_DELAY)
        }

        let image_bytes = System::new("", DefaultRuntime).block_on(async {
            let client = ClientBuilder::new().build(
                ClientConfig::new()
                    .disable_timeout()
                    .set_response_payload_limit(usize::MAX),
            );

            let api_response = client.get(BING_JSON_API).send().await.unwrap();
            let api_bytes = api_response.body().await.unwrap();
            let api_bytes_str = String::from_utf8(api_bytes.to_vec()).unwrap();
            let image_url_base = unsafe {
                sonic_rs::get_from_str_unchecked(&api_bytes_str, pointer!["images", 0, "urlbase"])
                    .unwrap()
            };
            let image_url = format!(
                "https://cn.bing.com{}_UHD.jpg",
                image_url_base.as_str().unwrap()
            );
            let image_response = client.get(image_url).send().await.unwrap();
            image_response.body().await.unwrap()
        });

        let mut file = fs::File::create(&wallpaper_path).unwrap();
        file.write_all(&image_bytes).unwrap();
        file.sync_all().unwrap();
    } else {
        #[cfg(target_os = "linux")]
        std::process::exit(1)
    }

    set_wallpaper(&wallpaper_path)
}
