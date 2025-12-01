use std::path::PathBuf;

use super::prelude::*;
use crate::{
    config::{CONFIG_FILE_NAME, DB_FILE_NAME},
    router::response::error::system::{BackupManagerErrKind, UnSafeErrKind},
    utils::SqlitePath,
};
use chrono::Datelike;
use rand::distr::Alphanumeric;
use sea_orm::{ConnectOptions, Database, DatabaseConnectionType, Delete};
use tauri_plugin_axum::AxumExt;
use ts_rs::TS;
use zip::write::SimpleFileOptions;

pub mod data_manage;
pub mod dataloader;

type R = Result<AppResponse<serde_json::Value>, SystemErrKind>;

pub fn router(s: ComplexState<impl Runtime>) -> Router {
    Router::new()
        .nest("/dataloader", dataloader::router())
        .nest("/data_manager", data_manage::router())
        .route("/time", get(get_sys_time))
        .route("/open_csv", post(open_csv))
        .route("/open_data_dir", post(open_data_dir))
        .route("/open_git_repo", post(open_git_repo))
        .route("/open_docs", post(open_docs))
        .route("/open_debug", post(open_debug))
        .route("/open_QuickReference", post(open_quick_reference))
        .route("/open_timedate_cpl", post(open_time_date_cpl))
        .route("/open_devtools", post(open_devtools))
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
    open::that(app.state::<Config>().read().await.export_path.clone())
        .map_err(SystemErrKind::OpenCsvFailed)?;

    ok!(())
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

    tokio::fs::remove_file(data_dir.join(DB_FILE_NAME))
        .await
        .map_err(UnSafeErrKind::ClearDbFileFailed)?;

    {
        let ctx = app.state::<BootloaderContex>();
        let mut ctx = ctx.write().await;
        ctx.db_ready = false;
    }

    ok!(())
}

async fn open_devtools(State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>) -> R {
    if let Some(main) = app.get_webview_window("main") {
        main.open_devtools();
        return ok!(());
    }
    Err(SystemErrKind::MainNotFound)
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
    State(ComplexState { ref db, .. }): State<ComplexState<impl Runtime>>,
    Json(UnsafeInvokeVertify { secret }): Json<UnsafeInvokeVertify>,
) -> R {
    if secret != env!("UNSAFE_INVOKE_SECRET") {
        return Err(SystemErrKind::UnsafeInvokeRejected);
    }

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

    if matches!(
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

    tokio::fs::remove_dir_all(&data_dir)
        .await
        .map_err(UnSafeErrKind::ClearDataDirFailed)?;

    tokio::fs::create_dir_all(&data_dir)
        .await
        .map_err(UnSafeErrKind::ReCreateDataDirFailed)?;

    {
        let ctx = app.state::<BootloaderContex>();
        let mut ctx = ctx.write().await;
        ctx.db_ready = false;
        ctx.conf_ready = false;
    }

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
    if !matches!(
        db,
        DatabaseConnection {
            inner: DatabaseConnectionType::Disconnected,
            ..
        }
    ) {
        db.close_by_ref()
            .await
            .map_err(BackupManagerErrKind::CloseDbError)?;
    }

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(BackupManagerErrKind::GetDataDir)?;

    let zip_file = std::fs::File::open(path).map_err(BackupManagerErrKind::OpenBackupFile)?;

    let mut zip_file =
        zip::ZipArchive::new(zip_file).map_err(BackupManagerErrKind::ReadBackupFile)?;

    if !zip_file.file_names().any(|name| name.eq(DB_FILE_NAME)) {
        return Err(BackupManagerErrKind::InvalidBackupFile(format!(
            "备份文件中缺少数据库文件: {}",
            DB_FILE_NAME
        )))?;
    };

    if !zip_file.file_names().any(|name| name.eq(CONFIG_FILE_NAME)) {
        return Err(BackupManagerErrKind::InvalidBackupFile(format!(
            "备份文件中不应包含配置文件: {}",
            CONFIG_FILE_NAME
        )))?;
    }

    for i in 0..zip_file.len() {
        let mut file = zip_file
            .by_index(i)
            .map_err(BackupManagerErrKind::ReadBackupFile)?;

        let out_path = file
            .enclosed_name()
            .ok_or_else(|| BackupManagerErrKind::InvalidBackupFile(file.name().to_string()))?;

        let out_path = data_dir.join(out_path);

        if file.is_dir() {
            tokio::fs::create_dir_all(out_path)
                .await
                .map_err(BackupManagerErrKind::ExtractBackupFile)?;
        } else {
            if let Some(p) = out_path.parent()
                && !p.exists()
            {
                tokio::fs::create_dir_all(p)
                    .await
                    .map_err(BackupManagerErrKind::ExtractBackupFile)?;
            }

            std::fs::File::create(&out_path)
                .and_then(|mut outfile| std::io::copy(&mut file, &mut outfile))
                .map_err(BackupManagerErrKind::ExtractBackupFile)?;
        }
    }

    let db_path = data_dir.join(DB_FILE_NAME);

    let opt: ConnectOptions = SqlitePath(db_path).into();

    let db_path = Database::connect(opt)
        .await
        .map_err(BackupManagerErrKind::DbConnect)?;

    let conf_path = data_dir.join(CONFIG_FILE_NAME);

    let conf: ConfigInner = serde_json::from_str(
        &tokio::fs::read_to_string(conf_path)
            .await
            .map_err(BackupManagerErrKind::ReadConfigFile)?,
    )
    .map_err(BackupManagerErrKind::DeserializeConfigFile)?;

    let conf_ = app.state::<Config>().clone_();

    {
        *conf_.write().await = conf;
    }

    app.set_router(crate::router::app(conf_, db_path, app.clone()))
        .await;

    {
        let ctx = app.state::<BootloaderContex>();
        ctx.write().await.db_ready = true;
    }

    ok!(())
}

pub async fn create_backup(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
) -> R {
    if !matches!(
        db,
        DatabaseConnection {
            inner: DatabaseConnectionType::Disconnected,
            ..
        }
    ) {
        db.close_by_ref()
            .await
            .map_err(BackupManagerErrKind::CloseDbError)?;
    }

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(BackupManagerErrKind::GetDataDir)?;

    let desktop_dir = app
        .path()
        .desktop_dir()
        .map_err(BackupManagerErrKind::GetDesktopDir)?;

    use rand::RngExt;
    let random_suffix: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(6)
        .collect();

    let dst_file_path = format!(
        "chrm-rev_{}_{}.zip",
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"),
        random_suffix,
    );

    let dst_file = desktop_dir.join(&dst_file_path);

    let mut dst_file =
        std::fs::File::create_new(dst_file).map_err(BackupManagerErrKind::CreateFile)?;

    let options = SimpleFileOptions::default();

    let walkdir = walkdir::WalkDir::new(data_dir.clone());

    let mut zip = zip::ZipWriter::new(&mut dst_file);

    for entry in walkdir.into_iter() {
        let entry = entry.map_err(BackupManagerErrKind::WalkDir)?;

        let path = entry.path();

        let path_stripped = path
            .strip_prefix(data_dir.clone())
            .map_err(BackupManagerErrKind::StripPrefixError)?;

        let path_as_string = path_stripped
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| BackupManagerErrKind::NotUtf8Path(path.to_path_buf()))?;

        if path.is_file() {
            zip.start_file(path_as_string, options)
                .map_err(BackupManagerErrKind::ZipError)?;
            let mut f = std::fs::File::open(path).map_err(BackupManagerErrKind::BackupFile)?;
            std::io::copy(&mut f, &mut zip).map_err(BackupManagerErrKind::BackupFile)?;
        } else if !path_stripped.as_os_str().is_empty() {
            zip.add_directory(path_as_string, options)
                .map_err(BackupManagerErrKind::ZipError)?;
        }
    }

    zip.finish().map_err(BackupManagerErrKind::ZipError)?;

    // 重连
    let db_path = data_dir.join(DB_FILE_NAME);

    if !db_path.exists() {
        return Err(BackupManagerErrKind::DbfileNotFound)?;
    }

    let opt: ConnectOptions = SqlitePath(db_path).into();

    let db_path = Database::connect(opt)
        .await
        .map_err(BackupManagerErrKind::DbConnect)?;

    let conf = app.state::<Config>().clone_();

    app.set_router(crate::router::app(conf, db_path, app.clone()))
        .await;

    ok!(dst_file_path)
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
    State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>,
    Json(config): Json<ConfigInner>,
) -> R {
    config
        .save(&app)
        .await
        .map_err(|e| SystemErrKind::PutConfig(e))?;

    {
        let conf = app.state::<Config>();
        *conf.write().await = config;
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
