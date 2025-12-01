use axum::response::Response;
use axum::{Json, response::IntoResponse};
use serde::Serialize;

pub mod error;

pub struct AppResponse<T> {
    pub status: axum::http::StatusCode,
    pub data: T,
}

impl<T: Serialize> IntoResponse for AppResponse<T> {
    fn into_response(self) -> Response {
        (self.status, Json(self.data)).into_response()
    }
}
