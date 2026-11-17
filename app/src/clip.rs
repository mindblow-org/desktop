use std::{fs, path::PathBuf};

use tauri::{Url, ipc::Response};

const PICS: [&str; 7] = ["png", "jpg", "jpeg", "webp", "gif", "bmp", "avif"];
const MAX: u64 = 50 << 20;

#[tauri::command]
pub fn clip(uri: String) -> Result<Response, String> {
    let path = Url::parse(&uri).ok().and_then(|u| u.to_file_path().ok()).ok_or("caminho inválido")?;
    let pic = path.extension().and_then(|e| e.to_str()).is_some_and(|e| PICS.contains(&e.to_lowercase().as_str()));
    if !pic || !held().contains(&path) {
        return Err("a imagem não está no clipboard".into());
    }
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > MAX {
        return Err("imagem grande demais".into());
    }
    fs::read(&path).map(Response::new).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pic() -> Result<Response, String> {
    png().map(Response::new).ok_or_else(|| "sem imagem no clipboard".into())
}

#[cfg(target_os = "linux")]
fn held() -> Vec<PathBuf> {
    gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD)
        .wait_for_uris()
        .iter()
        .filter_map(|u| Url::parse(u).ok()?.to_file_path().ok())
        .collect()
}

#[cfg(target_os = "linux")]
fn png() -> Option<Vec<u8>> {
    let d = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD).wait_for_contents(&gtk::gdk::Atom::intern("image/png"))?;
    let b = d.data();
    (!b.is_empty() && b.len() as u64 <= MAX).then_some(b)
}

#[cfg(not(target_os = "linux"))]
fn held() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(not(target_os = "linux"))]
fn png() -> Option<Vec<u8>> {
    None
}
