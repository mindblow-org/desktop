#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs, path::PathBuf};

use tauri::{Url, WebviewUrl, WebviewWindowBuilder};

const LOCAL: &str = "http://localhost:7341";

fn main() {
    let url = target();
    tauri::Builder::default()
        .setup(move |app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url.clone()))
                .title("mindblow")
                .inner_size(1280.0, 820.0)
                .min_inner_size(420.0, 480.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("mindblow");
}

fn target() -> Url {
    env::var("MINDBLOW_URL")
        .ok()
        .or_else(|| fs::read_to_string(conf()).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| LOCAL.parse().expect("url"))
}

fn conf() -> PathBuf {
    env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_default()
        .join("mindblow/url")
}
