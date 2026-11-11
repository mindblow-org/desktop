mod fail;
mod guard;
mod images;
mod items;
mod notes;
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

pub fn router(a: App) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/notes", get(notes::list).post(notes::add))
        .route("/api/notes/{id}", get(notes::get).patch(notes::edit).delete(notes::remove))
        .route("/api/notes/{id}/items", post(items::push))
        .route("/api/notes/{id}/items/{item}", patch(items::mark).delete(items::pull))
        .route("/api/notes/{id}/images", post(images::attach).layer(DefaultBodyLimit::max(MAX + 1)))
        .route("/api/notes/{id}/images/{img}", delete(images::detach))
        .route("/api/images/{img}", get(images::show))
        .route("/api/live", get(ws::live))
        .fallback(web::serve)
        .layer(middleware::from_fn_with_state(a.clone(), guard::check))
        .with_state(a)
}

async fn health(State(a): State<App>) -> Json<Value> {
    Json(json!({ "ok": true, "version": env!("CARGO_PKG_VERSION"), "link": a.link.as_deref() }))
}
