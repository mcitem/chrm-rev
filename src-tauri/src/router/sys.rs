use std::path::PathBuf;

use super::prelude::*;
use crate::router::response::error::system::{BackupManagerErrKind, UnSafeErrKind};
use chrono::Datelike;
use sea_orm::{DatabaseConnectionType, Delete};
use ts_rs::TS;

pub mod data_manage;
pub mod dataloader;
mod export_file;
mod maintenance;

type R = Result<AppResponse<serde_json::Value>, SystemErrKind>;

pub fn router(s: ComplexState<impl Runtime>) -> Router {
    Router::new()
        .nest("/dataloader", dataloader::router())
        .nest("/data_manager", data_manage::router())
        .route("/time", get(get_sys_time))
        .route("/open_csv", post(open_csv))
        .route("/export_file/create", post(export_file::create))
        .route("/export_file/path", put(export_file::save))
        .route("/open_data_dir", post(open_data_dir))
        .route("/open_git_repo", post(open_git_repo))
        .route("/open_docs", post(open_docs))
        .route("/open_debug", post(open_debug))
        .route("/open_QuickReference", post(open_quick_reference))
        .route("/open_timedate_cpl", post(open_time_date_cpl))
        .route("/open_devtools", post(open_devtools))
        .route("/open_logs", post(open_logs))
        .route("/config", put(put_config).get(get_config))
        .route("/validate/config", post(validate_config))
        .route("/bootloader/context", get(use_bootloader_context))
        .route("/backup/create", post(create_backup))
        .route("/backup/rollback", post(rollback_backup))
        .route("/unsafe/clear_data_dir", post(clear_data_dir))
        .route("/unsafe/clear_db_file", post(clear_db_file))
        .route("/unsafe/clear_record", post(clear_record))
        .with_state(s)
}

async fn open_csv(State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>) -> R {
    let path = app.state::<Config>().read().await.export_path.clone();
    if !export_file::is_available(&path)
        .await
        .map_err(SystemErrKind::OpenCsvFailed)?
    {
        return ok!({ "needs_configuration": true });
    }
    open::that(path).map_err(SystemErrKind::OpenCsvFailed)?;

    ok!({ "needs_configuration": false })
}

async fn open_git_repo() -> R {
    open::that("https://github.com/mcitem/chrm-rev").map_err(SystemErrKind::OpenGitRepoFailed)?;
    ok!(())
}

async fn open_data_dir(State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>) -> R {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(SystemErrKind::GetDataDir)?;
    open::that(data_dir).map_err(SystemErrKind::OpenDataDirFailed)?;
    ok!(())
}

pub async fn clear_db_file(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
    Json(UnsafeInvokeVertify { secret }): Json<UnsafeInvokeVertify>,
) -> R {
    if secret != env!("UNSAFE_INVOKE_SECRET") {
        return Err(SystemErrKind::UnsafeInvokeRejected);
    }

    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    maintenance::reset_business_data(&db)
        .await
        .map_err(UnSafeErrKind::ClearRecordFailed)?;
    app.state::<BootloaderContex>().write().await.db_ready = false;

    ok!(())
}

async fn open_devtools(State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>) -> R {
    if let Some(main) = app.get_webview_window("main") {
        main.open_devtools();
        return ok!(());
    }
    Err(SystemErrKind::MainNotFound)
}

async fn open_logs(State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>) -> R {
    let data_dir = app
        .path()
        .app_log_dir()
        .map_err(SystemErrKind::GetDataDir)?;
    open::that(data_dir).map_err(SystemErrKind::OpenLogsFailed)?;
    ok!(())
}

async fn open_time_date_cpl() -> R {
    use std::os::windows::process::CommandExt;
    // https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    std::process::Command::new("cmd")
        .args(&["/C", "start", "timedate.cpl"])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(SystemErrKind::OpenTimedateCpl)?;
    ok!(())
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "./sys.ts")]
pub struct UnsafeInvokeVertify {
    secret: String,
}

async fn clear_record(
    State(ComplexState { ref db, app }): State<ComplexState<impl Runtime>>,
    Json(UnsafeInvokeVertify { secret }): Json<UnsafeInvokeVertify>,
) -> R {
    if secret != env!("UNSAFE_INVOKE_SECRET") {
        return Err(SystemErrKind::UnsafeInvokeRejected);
    }

    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    Delete::many(crate::entity::record::Entity)
        .exec(db)
        .await
        .map_err(UnSafeErrKind::ClearRecordFailed)?;

    ok!(())
}

async fn open_debug(State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>) -> R {
    if let Some(debug) = app.get_webview_window("debug") {
        debug
            .show()
            .map_err(|e| SystemErrKind::OpenDocs(e, "debug.show"))?;
        debug
            .set_focus()
            .map_err(|e| SystemErrKind::OpenDocs(e, "debug.set_focus"))?;
        return ok!(());
    }

    tauri::WebviewWindow::builder(&app, "debug", tauri::WebviewUrl::App("/debug".into()))
        .build()
        .map_err(|e| SystemErrKind::OpenDocs(e, "debug.build"))?;

    ok!(())
}

async fn open_quick_reference(
    State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>,
) -> R {
    if let Some(docs) = app.get_webview_window("QuickReference") {
        docs.show()
            .map_err(|e| SystemErrKind::OpenDocs(e, "QuickReference.show"))?;
        docs.set_focus()
            .map_err(|e| SystemErrKind::OpenDocs(e, "QuickReference.set_focus"))?;
        return ok!(());
    }

    tauri::WebviewWindow::builder(
        &app,
        "QuickReference",
        tauri::WebviewUrl::App("/QuickReference".into()),
    )
    .title("QuickReference")
    .inner_size(1000_f64, 800_f64)
    .build()
    .map_err(|e| SystemErrKind::OpenDocs(e, "QuickReference.build"))?;
    ok!(())
}

async fn get_sys_time() -> R {
    let now = chrono::Local::now();
    let year = now.year();
    ok!({
        "time": now.format("%Y-%m-%d %H:%M:%S").to_string(),
        "date": now.format("%Y/%m/%d").to_string(),
        "year": year,
    })
}

pub async fn clear_data_dir(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
    Json(UnsafeInvokeVertify { secret }): Json<UnsafeInvokeVertify>,
) -> R {
    if secret != env!("UNSAFE_INVOKE_SECRET") {
        return Err(SystemErrKind::UnsafeInvokeRejected);
    }

    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    if !matches!(
        db,
        DatabaseConnection {
            inner: DatabaseConnectionType::Disconnected,
            ..
        }
    ) {
        match db.ping().await {
            Ok(()) => {
                db.close_by_ref()
                    .await
                    .map_err(UnSafeErrKind::CloseDbError)?;
            }
            Err(_) => {}
        };
    }

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(UnSafeErrKind::GetDataDir)?;

    if let Err(error) = tokio::fs::remove_dir_all(&data_dir).await {
        let path = data_dir.join(crate::config::DB_FILE_NAME);
        if path.exists() {
            let recovered = maintenance::connect(path)
                .await
                .map_err(BackupManagerErrKind::DbConnect)?;
            maintenance::reload(&app, recovered).await?;
        }
        return Err(UnSafeErrKind::ClearDataDirFailed(error).into());
    }
    tokio::fs::create_dir_all(&data_dir)
        .await
        .map_err(UnSafeErrKind::ReCreateDataDirFailed)?;
    let fresh = maintenance::connect(data_dir.join(crate::config::DB_FILE_NAME))
        .await
        .map_err(BackupManagerErrKind::DbConnect)?;
    use sea_orm_migration::MigratorTrait;
    crate::Migrator::up(&fresh, None)
        .await
        .map_err(BackupManagerErrKind::DbConnect)?;
    maintenance::reload(&app, fresh).await?;

    ok!(())
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./sys.ts")]
struct RollBackBackup {
    path: PathBuf,
}

async fn rollback_backup(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
    Json(RollBackBackup { path }): Json<RollBackBackup>,
) -> R {
    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(BackupManagerErrKind::GetDataDir)?;
    maintenance::restore(&app, &db, path, data_dir).await?;
    ok!(())
}

pub async fn create_backup(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
) -> R {
    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(BackupManagerErrKind::GetDataDir)?;
    let desktop = app
        .path()
        .desktop_dir()
        .map_err(BackupManagerErrKind::GetDesktopDir)?;
    let name = format!(
        "chrm-rev_{}_{:032x}.db",
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"),
        rand::random::<u128>()
    );
    maintenance::backup(&db, data_dir, desktop.join(&name)).await?;
    ok!(name)
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./sys.ts")]
struct OpenDocs {
    #[serde(default = "default_docs_url")]
    #[ts(as = "Option<String>")]
    url: String,
}

fn default_docs_url() -> String {
    String::from("docs/index.html")
}

async fn open_docs(
    State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>,
    Json(OpenDocs { url }): Json<OpenDocs>,
) -> R {
    if let Some(docs) = app.get_webview_window("docs") {
        docs.show()
            .map_err(|e| SystemErrKind::OpenDocs(e, "docs.show"))?;
        docs.set_focus()
            .map_err(|e| SystemErrKind::OpenDocs(e, "docs.set_focus"))?;
        return ok!(());
    }

    tauri::WebviewWindow::builder(&app, "docs", tauri::WebviewUrl::App(url.into()))
        .inner_size(800_f64, 600_f64)
        .build()
        .map_err(|e| SystemErrKind::OpenDocs(e, "docs.build"))?;
    ok!(())
}

async fn get_config(State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>) -> R {
    let conf = app.state::<Config>();
    ok!(conf.read().await.clone())
}

async fn put_config(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
    Json(config): Json<ConfigInner>,
) -> R {
    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    {
        // Keep concurrent database writes and cache updates in the same order.
        let conf = app.state::<Config>();
        let mut current = conf.write().await;
        config
            .save(&db)
            .await
            .map_err(|e| SystemErrKind::PutConfig(Box::new(e)))?;
        *current = config;
    }
    {
        let ctx = app.state::<BootloaderContex>();
        ctx.write().await.conf_ready = true;
    };

    ok!(())
}

pub async fn validate_config(Json(c): Json<ConfigInner>) -> R {
    ok!(c)
}

async fn use_bootloader_context(
    State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>,
) -> R {
    let ctx = *app.state::<BootloaderContex>().read().await;
    ok!(ctx)
}
