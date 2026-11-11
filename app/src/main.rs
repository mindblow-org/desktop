#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs, io::ErrorKind, path::PathBuf};

use server::{Config, spawn};
use tauri::{Url, WebviewUrl, WebviewWindowBuilder};

const LOCAL: &str = "http://localhost:7341";

fn main() {
    node();
    let url: Url = LOCAL.parse().expect("url");
    tauri::Builder::default()
        .setup(move |app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url.clone()))
                .title("mindblow")
                .inner_size(1280.0, 820.0)
                .min_inner_size(420.0, 480.0)
                .decorations(false)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("mindblow");
}

/// O app leva o mindblow junto: sobe o servidor em `localhost:7341` com o banco desta
/// máquina, então funciona sem rede. Se a porta já está ocupada, o serviço roda aqui
/// mesmo e a janela usa ele.
fn node() {
    let mut cfg = Config::env();
    cfg.peer = cfg.peer.or_else(peer);
    match spawn(cfg) {
        Err(e) if e.kind() != ErrorKind::AddrInUse => eprintln!("mindblow: {e}"),
        _ => {}
    }
}

/// O mindblow de outra máquina, para sincronizar: `MINDBLOW_URL` ou `~/.config/mindblow/url`.
/// Antes o app abria esse endereço direto; agora ele vira um link do sync.
fn peer() -> Option<String> {
    env::var("MINDBLOW_URL")
        .ok()
        .or_else(|| fs::read_to_string(conf()).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| s.parse::<Url>().is_ok_and(|u| !matches!(u.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))))
}

fn conf() -> PathBuf {
    env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_default()
        .join("mindblow/url")
}
