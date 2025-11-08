mod backup;
mod sudo;

pub use backup::*;
pub use sudo::*;

use tauri::{Manager, Runtime};

use crate::DATADIR;

// todo，为命令注入log

#[tauri::command]
pub async fn open_web_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_core_dir() -> Result<(), String> {
    let core_dir = tauri::utils::platform::current_exe()
        .map_err(|_| "Failed to get app local data directory")?;
    open::that(core_dir.parent().ok_or("Failed to get parent directory")?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_data_dir() -> Result<(), String> {
    open::that(DATADIR.as_path()).map_err(|e| e.to_string())
}

/// Appdata/Local/<id>/logs
#[tauri::command]
pub async fn open_logs_dir<R: Runtime>(app_handle: tauri::AppHandle<R>) -> Result<(), String> {
    open::that(app_handle.path().app_log_dir().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_devtools<R: Runtime>(app_handle: tauri::AppHandle<R>) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if !window.is_devtools_open() {
            window.open_devtools();
        } else {
            window.close_devtools();
        }
    }
}

#[tauri::command]
pub async fn open_timedate_cpl() -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(&["/C", "start", "timedate.cpl"])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
