use crate::config::Config;
use crate::config::ConfigInner;
use crate::config::DB_FILE_NAME;
use crate::utils::SqlitePath;
use sea_orm::ConnectOptions;
use sea_orm::Database;
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

    // println!("data dir: {:?}", data_dir);
    // println!("cache dir: {:?}", cache_dir);

    app_handle.plugin(tauri_plugin_pinia::Builder::new().path(cache_dir).build())?;

    std::fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join(DB_FILE_NAME);
    let previous = data_dir.join(format!("{DB_FILE_NAME}.previous"));
    if !db_path.exists() && previous.exists() {
        std::fs::rename(previous, &db_path)?;
    }
    let (db, loaded_conf, db_ready) = tauri::async_runtime::block_on(async {
        let opt: ConnectOptions = SqlitePath(db_path).into();
        let db = Database::connect(opt).await?;
        // Configuration must be writable before the data import wizard starts.
        crate::Migrator::up(&db, None).await?;
        let db_ready = crate::utils::data_ready(&db).await?;
        let loaded_conf = ConfigInner::load(&db).await?;
        Ok::<_, sea_orm::DbErr>((db, loaded_conf, db_ready))
    })?;
    let conf_ready = loaded_conf.is_some();
    let conf = Config::new(loaded_conf.unwrap_or_default());

    app.manage(conf.clone());

    app.block_set_router(crate::router::app(conf, db, app_handle.clone()));

    let inner = BootloaderContextInner {
        conf_ready,
        db_ready,
    };
    // println!("bootloader context: {:?}", inner);

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
