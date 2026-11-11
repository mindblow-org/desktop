use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use domain::{Draft, Id, Note, Patch};
use serde::Deserialize;

use super::fail::Res;
use crate::App;

#[derive(Deserialize)]
pub struct Find {
    q: Option<String>,
}

pub async fn list(State(a): State<App>, Query(f): Query<Find>) -> Res<Json<Vec<Note>>> {
    Ok(Json(match f.q {
        Some(q) => a.notes.find(&q)?,
        None => a.notes.list()?,
    }))
}

pub async fn get(State(a): State<App>, Path(id): Path<Id>) -> Res<Json<Note>> {
    Ok(Json(a.notes.get(id)?))
}

pub async fn add(State(a): State<App>, Json(d): Json<Draft>) -> Res<(StatusCode, Json<Note>)> {
    Ok((StatusCode::CREATED, Json(a.notes.add(d)?)))
}

pub async fn edit(State(a): State<App>, Path(id): Path<Id>, Json(p): Json<Patch>) -> Res<Json<Note>> {
    Ok(Json(a.notes.edit(id, p)?))
}

pub async fn remove(State(a): State<App>, Path(id): Path<Id>) -> Res<StatusCode> {
    a.notes.remove(id)?;
    Ok(StatusCode::NO_CONTENT)
}
