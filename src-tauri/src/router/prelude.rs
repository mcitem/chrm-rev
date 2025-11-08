// axum router
pub use axum::routing::{delete, put};
pub use axum::routing::{get, post};
//提取器
pub use axum::extract::{Path, State};

// 状态
pub(super) use super::state::AppState;

// 数据库
pub use sea_orm::Set;
pub use sea_orm::prelude::*;

pub use crate::entity::item;

// 响应
pub use super::response::R;
pub use crate::ok;

pub use serde::Deserialize;
pub use ts_rs::TS;

pub type Router<T = AppState> = axum::Router<T>;

pub use crate::config::Config;
