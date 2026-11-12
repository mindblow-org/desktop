mod fail;
mod guard;
mod images;
mod items;
mod links;
mod notes;
mod sync;
mod ws;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    middleware,
    routing::{delete, get, patch, post},
};
use domain::image::MAX;
use serde_json::{Value, json};

use crate::{App, web};

/// Um lote de sync: até 50 notas, e um board pode ter 1 MB.
const BATCH: usize = 64 * 1024 * 1024;

pub fn router(a: App) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/notes", get(notes::list).post(notes::add))
        .route("/api/notes/{id}", get(notes::get).patch(notes::edit).delete(notes::remove))
        .route("/api/notes/{id}/items", post(items::push))
        .route("/api/notes/{id}/items/{item}", patch(items::mark).delete(items::pull))
        .route("/api/notes/{id}/images", post(images::attach).layer(DefaultBodyLimit::max(MAX + 1)))
        .route("/api/notes/{id}/images/{img}", delete(images::detach))
        .route("/api/images/{img}", get(images::show).put(images::adopt).layer(DefaultBodyLimit::max(MAX + 1)))
        .route("/api/sync", get(sync::page).post(sync::take).layer(DefaultBodyLimit::max(BATCH)))
        .route("/api/sync/now", post(sync::now))
        .route("/api/links", get(links::list).post(links::add).delete(links::remove))
        .route("/api/live", get(ws::live))
        .fallback(web::serve)
        .layer(middleware::from_fn_with_state(a.clone(), guard::check))
        .with_state(a)
}

async fn health(State(a): State<App>) -> Json<Value> {
    Json(json!({ "ok": true, "version": env!("CARGO_PKG_VERSION"), "link": a.link.as_deref(), "node": a.node }))
}
