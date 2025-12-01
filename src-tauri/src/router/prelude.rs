pub use super::response::{
    AppResponse,
    error::{AppErr, biz::BizErrKind, system::SystemErrKind},
};
pub use super::utils::*;
pub use crate::config::{Config, ConfigInner};
pub use crate::entity::*;
pub use crate::ok;
pub use crate::router::state::AppState;
pub use crate::router::state::ComplexState;
pub use crate::ui::setup::BootloaderContex;
pub use axum::extract::State;
pub use axum::extract::{Path, Query};
pub use axum::{
    Json, Router,
    routing::{get, post, put},
};
pub use sea_orm::Condition;
pub use sea_orm::DbConn;
pub use sea_orm::QueryOrder;
pub use sea_orm::prelude::*;
pub use sea_orm::sea_query::SimpleExpr;
pub use serde::{Deserialize, Serialize};
pub use std::str::FromStr;
pub use tauri::Manager;
pub use tauri::Runtime;
pub use ts_rs::TS;

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

pub type R<T = serde_json::Value, E = AppErr> = Result<AppResponse<T>, E>;
