use std::path::PathBuf;

use axum::Json;
use chrono::Datelike;

use crate::{config::CONFIG, router::response::AppErr};

use super::prelude::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/time", get(get_sys_time))
        .route("/export_all", post(export_all))
        .route("/config", get(get_config))
        .route("/config/reload", get(reload_conf))
        .route(
            "/config/exportPath",
            post(set_export_path).delete(clear_export_path),
        )
        .route(
            "/config/AdvanceExportPath",
            post(set_advance_export_path).delete(clear_advance_export_path),
        )
}

async fn export_all(State(ref db): State<DbConn>, Json(payload): Json<SetExportPath>) -> R {
    let res = crate::router::utils::export::export_all(db, &payload.path).await;
    res.map_err(|e| AppErr::e_400(&format!("导出失败: {}", e)))?;

    ok!(())
}

async fn reload_conf() -> R {
    Config::reload().await;
    ok!(())
}

async fn set_advance_export_path(Json(payload): Json<SetExportPath>) -> R {
    let path = PathBuf::from(&payload.path);

    if path.is_dir() {
        return Err(AppErr::e_400("路径不是一个文件"));
    }

    if path.is_relative() {
        return Err(AppErr::e_400("路径不是一个绝对路径"));
    }

    if path.extension() != Some(std::ffi::OsStr::new("xlsx")) {
        return Err(AppErr::e_400("路径不是xlsx文件"));
    }

    Config::set_advance_export_path(Some(payload.path)).await;
    CONFIG.save().await.ok();

    ok!(())
}

async fn clear_advance_export_path() -> R {
    Config::set_advance_export_path(None).await;
    CONFIG.save().await.ok();
    ok!(())
}

/// 获取系统当前时间
/// 也用于签名导出
async fn get_sys_time() -> R {
    let now = chrono::Local::now();
    let year = now.year();

    let conf = Config::get_date_format().await.unwrap_or("%Y/%m/%d".into());

    ok!({
        "time": now.format("%Y-%m-%d %H:%M:%S").to_string(),
        "date": now.format(&conf).to_string(),
        "year": year,
    })
}

async fn get_config() -> R {
    let cf = Config::conf().await;
    ok!(cf)
}

#[derive(serde::Deserialize)]
struct SetExportPath {
    path: String,
}

async fn clear_export_path() -> R {
    Config::set_export_path(None).await;
    CONFIG.save().await.ok();

    ok!(())
}

async fn set_export_path(Json(payload): Json<SetExportPath>) -> R {
    let pathtest = std::path::Path::new(&payload.path);

    if pathtest.is_dir() {
        return Err(AppErr::e_400("路径不是一个文件"));
    }

    if pathtest.is_relative() {
        return Err(AppErr::e_400("路径不是一个绝对路径"));
    }

    if pathtest.extension() != Some(std::ffi::OsStr::new("csv")) {
        return Err(AppErr::e_400("路径不是一个csv文件"));
    }

    Config::set_export_path(Some(payload.path)).await;

    CONFIG.save().await.ok();

    ok!(())
}
