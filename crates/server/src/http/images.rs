use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query, State},
    http::{
        HeaderMap, StatusCode,
        header::{CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS},
    },
    response::{IntoResponse, Response},
};
use domain::{Id, Note};
use serde::Deserialize;

use super::fail::Res;
use crate::App;

#[derive(Deserialize)]
pub struct Size {
    #[serde(default)]
    w: u32,
    #[serde(default)]
    h: u32,
}

pub async fn attach(
    State(a): State<App>,
    Path(id): Path<Id>,
    Query(s): Query<Size>,
    h: HeaderMap,
    body: Bytes,
) -> Res<(StatusCode, Json<Note>)> {
    Ok((StatusCode::CREATED, Json(a.notes.attach(id, &mime(&h), &body, s.w, s.h)?)))
}

/// Imagem que veio de outro aparelho, com o id que a nota de lá já usa.
pub async fn adopt(State(a): State<App>, Path(img): Path<Id>, h: HeaderMap, body: Bytes) -> Res<StatusCode> {
    a.notes.adopt(img, &mime(&h), &body)?;
    Ok(StatusCode::NO_CONTENT)
}

fn mime(h: &HeaderMap) -> String {
    h.get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

pub async fn detach(State(a): State<App>, Path((id, img)): Path<(Id, Id)>) -> Res<Json<Note>> {
    Ok(Json(a.notes.detach(id, img)?))
}

pub async fn show(State(a): State<App>, Path(img): Path<Id>) -> Res<Response> {
    let (mime, data) = a.notes.image(img)?;
    Ok((
        [
            (CONTENT_TYPE, mime),
            (CACHE_CONTROL, "private, max-age=31536000, immutable".into()),
            (X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
            (CONTENT_SECURITY_POLICY, "default-src 'none'; sandbox".into()),
        ],
        data,
    )
        .into_response())
}
