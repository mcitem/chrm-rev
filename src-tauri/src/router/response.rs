use axum::response::Response;
use axum::{Json, http::StatusCode, response::IntoResponse};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
#[macro_export(local_inner_macros)]
macro_rules! ok {
    ($err:ty, $($json:tt)*) => {
        Ok::<_, $err>($crate::router::response::AppResponse {
            status: axum::http::StatusCode::OK,
            data: serde_json::json!($($json)*),
        })
    };
    ($err:ty; $($json:tt)*) => {
        Ok::<_, $err>($crate::router::response::AppResponse {
            status: axum::http::StatusCode::OK,
            data: serde_json::json!($($json)*),
        })
    };
    ($($json:tt)*) => {
        Ok::<_, _>($crate::router::response::AppResponse {
            status: axum::http::StatusCode::OK,
            data: serde_json::json!($($json)*),
        })
    };
}

#[derive(Default)]

pub struct AppResponse<T> {
    pub status: StatusCode,
    pub data: T,
}

// impl<T> AppResponse<T> {
//     pub fn map<E: IntoResponse>(self) -> Result<Self, E> {
//         Ok(self)
//     }
// }

impl<T> From<T> for AppResponse<T>
where
    T: Serialize,
{
    fn from(data: T) -> Self {
        AppResponse {
            status: StatusCode::OK,
            data,
        }
    }
}

impl<T: Serialize> IntoResponse for AppResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(json!({
                "data": self.data,
            })),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppErr {
    #[serde(skip)]
    pub status: StatusCode,
    pub code: u16,
    pub msg: Option<String>,
    pub error: Option<String>,
}
impl AppErr {
    pub fn map<T>(self) -> Result<T, Self> {
        Err(self)
    }
}

impl Default for AppErr {
    fn default() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            msg: Some("遇到未知错误".into()),
            error: None,
            code: 0,
        }
    }
}

impl IntoResponse for AppErr {
    fn into_response(self) -> Response {
        let status = self.status;
        (status, Json(self)).into_response()
    }
}
pub trait AppErrExt<T> {
    fn e_400(self) -> T;
    fn e_404(self) -> T;
}

impl AppErrExt<AppErr> for &str {
    fn e_400(self) -> AppErr {
        AppErr::e_400(self)
    }

    fn e_404(self) -> AppErr {
        AppErr::e_404(self)
    }
}

impl AppErrExt<AppErr> for String {
    fn e_400(self) -> AppErr {
        AppErr::e_400(self)
    }

    fn e_404(self) -> AppErr {
        AppErr::e_404(self)
    }
}

impl AppErr {
    pub fn e_404<T: ToString>(msg: T) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            msg: Some(msg.to_string()),
            code: 404,
            ..Default::default()
        }
    }

    pub fn e_400<T: ToString>(msg: T) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            msg: Some(msg.to_string()),
            code: 400,
            ..Default::default()
        }
    }

    // pub fn msg(self, msg: impl ToString) -> Self {
    //     Self {
    //         msg: Some(msg.to_string()),
    //         ..self
    //     }
    // }
}

impl From<DbErr> for AppErr {
    fn from(err: DbErr) -> Self {
        err.to_string().e_400()
    }
}

use calamine::Error;
impl From<Error> for AppErr {
    fn from(err: Error) -> Self {
        err.to_string().e_400()
    }
}

use sea_orm::ColumnFromStrErr;
impl From<ColumnFromStrErr> for AppErr {
    fn from(err: ColumnFromStrErr) -> Self {
        err.to_string().e_400()
    }
}

use crate::utils::ExportError;
impl From<ExportError> for AppErr {
    fn from(err: ExportError) -> Self {
        err.to_string().e_400()
    }
}

use rusqlite::Error as RSqliteError;
impl From<RSqliteError> for AppErr {
    fn from(err: RSqliteError) -> Self {
        err.to_string().e_400()
    }
}

/// 统一响应
pub type R<T = Value> = Result<AppResponse<T>, AppErr>;
