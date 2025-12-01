pub mod biz;
pub mod system;
pub mod utils;

use crate::{
    impl_app_error, impl_into_app_error,
    router::{
        prelude::SystemErrKind,
        response::error::{
            biz::BizErrKind,
            system::{BackupManagerErrKind, DataloaderErrKind, UnSafeErrKind},
        },
    },
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_with::skip_serializing_none;
use ts_rs::TS;
use utils::AppError;

#[derive(Debug, derive_more::From)]
pub enum AppErr {
    BizErr(BizErrKind),
    System(SystemErrKind),
}

impl_app_error!(
    for AppErr;
    @delegates;
    BizErr,
    System,
);

impl_into_app_error!(
    for AppErr;
    SystemErrKind => [DataloaderErrKind,BackupManagerErrKind,UnSafeErrKind,],
);

#[skip_serializing_none]
#[derive(Serialize, TS)]
#[ts(export, export_to = "./utils.ts")]
struct ErrorResponse {
    msg: Option<String>,
    error: String,
}

impl IntoResponse for AppErr {
    fn into_response(self) -> Response {
        (
            self.code(),
            Json(ErrorResponse {
                msg: self.msg(),
                error: self.error(),
            }),
        )
            .into_response()
    }
}
