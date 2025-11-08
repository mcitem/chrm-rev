use std::time::Duration;

use rusqlite::{Connection, backup::Backup};
use tauri::{AppHandle, Runtime};

use crate::DATADIR;

#[tauri::command]
pub async fn backup(sign: Option<String>) -> Result<(), String> {
    let name = if let Some(s) = sign {
        s
    } else {
        chrono::Local::now()
            .format("backup-%Y%m%d-%H%M%S")
            .to_string()
    };

    let dst_dir = DATADIR.join("backup");

    tokio::fs::create_dir_all(dst_dir.as_path())
        .await
        .map_err(|e| e.to_string())?;

    let dst = dst_dir.join(format!("{}.db", name));
    let result = tokio::task::spawn_blocking(move || {
        let src_conn = Connection::open(DATADIR.join("chrm-rev.db"))?;
        let mut dst_conn = Connection::open(dst)?;
        let backup = Backup::new(&src_conn, &mut dst_conn)?;
        backup.run_to_completion(
            10,
            Duration::from_millis(100),
            Some(|p| {
                log::debug!("Copied {} of {} pages...", p.remaining, p.pagecount);
            }),
        )?;
        Ok::<_, rusqlite::Error>(())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string());
    log::info!("备份数据库“{}”: 结果{:?}", name, result);
    result
}
