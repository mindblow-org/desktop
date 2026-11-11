use axum::{
    http::{
        StatusCode, Uri,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../web/dist"]
#[allow_missing = true]
struct Dist;

const MISSING: &str = "interface não compilada: rode pnpm build em web/";

pub async fn serve(uri: Uri) -> Response {
    let p = uri.path().trim_start_matches('/');
    if p.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let (p, f) = match Dist::get(p).filter(|_| !p.is_empty()) {
        Some(f) => (p, f),
        None => match Dist::get("index.html") {
            Some(f) => ("index.html", f),
            None => return (StatusCode::NOT_FOUND, MISSING).into_response(),
        },
    };
    let cache = if p.starts_with("assets/") { "public, max-age=31536000, immutable" } else { "no-cache" };
    ([(CONTENT_TYPE, f.metadata.mimetype().to_owned()), (CACHE_CONTROL, cache.to_owned())], f.data).into_response()
}
