use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use domain::Error;
use serde::Deserialize;

use super::fail::Res;
use crate::{App, sync};

#[derive(Deserialize)]
pub struct Add {
    url: String,
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct Url {
    url: String,
}

pub async fn list(State(a): State<App>) -> Json<Vec<sync::State>> {
    Json(a.sync.list())
}

pub async fn add(State(a): State<App>, Json(b): Json<Add>) -> Res<(StatusCode, Json<Vec<sync::State>>)> {
    let (url, token) = sync::parse(&b.url).ok_or(Error::Invalid("link inválido"))?;
    a.sync.add(&url, b.token.as_deref().unwrap_or(&token))?;
    Ok((StatusCode::CREATED, Json(a.sync.list())))
}

pub async fn remove(State(a): State<App>, Query(q): Query<Url>) -> Res<StatusCode> {
    if a.sync.remove(&q.url)? { Ok(StatusCode::NO_CONTENT) } else { Err(Error::Missing.into()) }
}
