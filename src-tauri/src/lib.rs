#![allow(unused)]

pub(crate) mod cmd;
pub mod config;
mod define;
mod entity;
pub mod migration;
pub mod router;
mod ui;
pub mod utils;

use sea_orm::{ConnectOptions, Database};
use std::{path::PathBuf, sync::LazyLock};
use ui::run_inner;
use utils::SqlitePath;

use crate::config::CONFIG;

const IDENTIFIER: &'static str = "com.mcitem.chrm-rev";

/// 共享数据目录路径
static DATADIR: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let current_exe = std::env::current_exe().unwrap();
            let fallback = current_exe.parent().unwrap().to_path_buf();
            fallback
        })
        .join(IDENTIFIER)
});

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 防止休眠
    unsafe {
        use windows_sys::Win32::System::Power;
        // https://learn.microsoft.com/zh-cn/windows/win32/api/winbase/nf-winbase-setthreadexecutionstate
        Power::SetThreadExecutionState(
            Power::ES_CONTINUOUS | Power::ES_DISPLAY_REQUIRED | Power::ES_SYSTEM_REQUIRED,
        );
    };

    // 固定webview版本
    // https://github.com/tauri-apps/tauri/issues/13817
    // unsafe {
    //     std::env::set_var(
    //         "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER",
    //         "./Microsoft.WebView2.FixedVersionRuntime.109.0.1518.78.x86/",
    //     );
    // };

    // 创建数据目录
    std::fs::create_dir_all(DATADIR.as_path())?;
    // 加载配置

    tauri::async_runtime::block_on(async { CONFIG.save().await })?;

    // 初始化数据库连接
    let db = tauri::async_runtime::block_on(async {
        let mut opt: ConnectOptions = SqlitePath(DATADIR.join("chrm-rev.db")).into();
        opt.sqlx_logging(false)
            .connect_timeout(std::time::Duration::from_secs(5));
        Database::connect(opt).await
    })?;

    // 挂载http插件
    let axum = tauri_plugin_axum::init(router::app(db));

    Ok(run_inner(tauri::Builder::default().plugin(axum))?)
}
