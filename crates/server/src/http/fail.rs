use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::Error;
use serde_json::json;

pub struct Fail(Error);

impl From<Error> for Fail {
    fn from(e: Error) -> Self {
        Self(e)
    }
}

impl IntoResponse for Fail {
    fn into_response(self) -> Response {
        let code = match self.0 {
            Error::Missing | Error::NoItem | Error::NoImage => StatusCode::NOT_FOUND,
            Error::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Error::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}

pub type Res<T> = Result<T, Fail>;
