use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use domain::{Id, Item, Note, item};

use super::fail::Res;
use crate::App;

pub async fn push(State(a): State<App>, Path(id): Path<Id>, Json(i): Json<Item>) -> Res<(StatusCode, Json<Note>)> {
    Ok((StatusCode::CREATED, Json(a.notes.push(id, i)?)))
}

pub async fn mark(
    State(a): State<App>,
    Path((id, i)): Path<(Id, Id)>,
    Json(p): Json<item::Patch>,
) -> Res<Json<Note>> {
    Ok(Json(a.notes.mark(id, i, p)?))
}

pub async fn pull(State(a): State<App>, Path((id, i)): Path<(Id, Id)>) -> Res<Json<Note>> {
    Ok(Json(a.notes.pull(id, i)?))
}
