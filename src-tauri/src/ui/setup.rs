use crate::config::CONFIG_FILE_NAME;
use crate::config::Config;
use crate::config::ConfigInner;
use crate::config::DB_FILE_NAME;
use crate::utils::SqlitePath;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm_migration::MigratorTrait;
use serde::Serialize;
use std::ops::Deref;
use tauri::Manager;
use tauri::Runtime;
use tauri::WebviewUrl;
use tauri::WebviewWindowBuilder as WWB;
use tauri_plugin_axum::AxumExt;
use tokio::sync::RwLock;
use ts_rs::TS;

pub struct BootloaderContex(pub RwLock<BootloaderContextInner>);

impl Deref for BootloaderContex {
    type Target = RwLock<BootloaderContextInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BootloaderContex {
    pub fn new(inner: BootloaderContextInner) -> Self {
        Self(RwLock::new(inner))
    }
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export, export_to = "./bootloader.ts")]
pub struct BootloaderContextInner {
    pub conf_ready: bool,
    pub db_ready: bool,
}

/// FnOnce(&mut App<R>) -> std::result::Result<(), Box<dyn std::error::Error>> + Send + 'static,
#[inline(always)]
pub fn setup<R>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>>
where
    R: Runtime,
{
    let app_handle = app.handle();

    let path_resolver = app_handle.path();

    let data_dir = path_resolver.app_data_dir()?;

    let cache_dir = path_resolver.app_cache_dir()?;

    println!("data dir: {:?}", data_dir);
    println!("cache dir: {:?}", cache_dir);

    app_handle.plugin(tauri_plugin_pinia::Builder::new().path(cache_dir).build())?;

    std::fs::create_dir_all(&data_dir)?;

    let conf_path = data_dir.join(CONFIG_FILE_NAME);

    let (conf, conf_ready) = if conf_path.exists()
        && let Ok(conf_str) = std::fs::read_to_string(&conf_path)
        && let Ok(conf) = serde_json::from_str::<ConfigInner>(&conf_str)
    {
        (Config::new(conf), true)
    } else {
        (Default::default(), false)
    };

    let db_path = data_dir.join(DB_FILE_NAME);

    let (db, db_ready) = if db_path.exists()
        && let Ok(db) = tauri::async_runtime::block_on(async move {
            let opt: ConnectOptions = SqlitePath(db_path).into();
            let db = Database::connect(opt).await?;
            let pending = crate::Migrator::get_pending_migrations(&db).await?;

            println!("pending migration: {}", pending.len());

            if pending.is_empty() {
                Ok::<(DatabaseConnection, bool), DbErr>((db, true))
            } else {
                Ok((db, false))
            }
        }) {
        db
    } else {
        (Default::default(), false)
    };

    app.manage(conf.clone());

    app.block_set_router(crate::router::app(conf, db, app_handle.clone()));

    let inner = BootloaderContextInner {
        conf_ready,
        db_ready,
    };
    println!("bootloader context: {:?}", inner);

    app.manage(BootloaderContex::new(inner));

    WWB::new(app, "main", WebviewUrl::App("/".into()))
        .title("Chrm Rev")
        .inner_size(810_f64, 600_f64)
        .min_inner_size(800_f64, 600_f64)
        .use_https_scheme(true)
        .initialization_script(format!(
            "window.UNSAFE_INVOKE_SECRET = \"{}\"",
            env!("UNSAFE_INVOKE_SECRET")
        ))
        .auto_resize()
        .build()?;

    #[cfg(debug_assertions)]
    app_handle
        .get_webview_window("main")
        .ok_or("main window not found")?
        .open_devtools();

    Ok(())
}
