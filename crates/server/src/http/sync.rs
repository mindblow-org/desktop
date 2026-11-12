use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use domain::{Gone, Shot, Took};
use serde::Deserialize;

use super::fail::Res;
use crate::{
    App,
    sync::{PAGE, Page},
};

#[derive(Deserialize)]
pub struct Since {
    #[serde(default)]
    since: i64,
    limit: Option<usize>,
}

pub async fn page(State(a): State<App>, Query(q): Query<Since>) -> Res<Json<Page>> {
    let limit = q.limit.unwrap_or(PAGE).clamp(1, PAGE);
    Ok(Json(Page { node: a.node, feed: a.notes.feed(q.since, limit)? }))
}

#[derive(Deserialize)]
pub struct Batch {
    #[serde(default)]
    notes: Vec<Shot>,
    #[serde(default)]
    gone: Vec<Gone>,
}

pub async fn take(State(a): State<App>, Json(b): Json<Batch>) -> Res<Json<Took>> {
    Ok(Json(a.notes.take(b.notes, b.gone)?))
}

pub async fn now(State(a): State<App>) -> StatusCode {
    a.sync.now();
    StatusCode::NO_CONTENT
}
