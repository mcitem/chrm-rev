use super::super::response::error::system::BackupManagerErrKind as Error;
use crate::{
    config::{Config, ConfigInner, DB_FILE_NAME},
    ui::setup::BootloaderContex,
    utils::{SqlitePath, data_ready},
};
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement, TransactionTrait,
};
use sea_orm_migration::MigratorTrait;
use std::path::{Path, PathBuf};
use tauri::{Manager, Runtime};
use tauri_plugin_axum::AxumExt;

fn task_error(error: tokio::task::JoinError) -> Error {
    Error::BackupFile(std::io::Error::other(error.to_string()))
}

pub async fn connect(path: PathBuf) -> Result<DatabaseConnection, DbErr> {
    let options: sea_orm::ConnectOptions = SqlitePath(path).into();
    Database::connect(options).await
}

pub async fn reset_business_data(db: &DatabaseConnection) -> Result<(), DbErr> {
    let txn = db.begin().await?;
    for table in ["record", "student", "item"] {
        txn.execute_unprepared(&format!("DELETE FROM {table}"))
            .await?;
    }
    crate::utils::set_data_ready(&txn, false).await?;
    txn.commit().await
}

struct LoadedDatabase {
    db: DatabaseConnection,
    config: Option<ConfigInner>,
    ready: bool,
}

async fn load_database(db: DatabaseConnection) -> Result<LoadedDatabase, Error> {
    let config = ConfigInner::load(&db).await.map_err(Error::DbConnect)?;
    let ready = data_ready(&db).await.map_err(Error::DbConnect)?;
    Ok(LoadedDatabase { db, config, ready })
}

async fn apply_loaded(app: &tauri::AppHandle<impl Runtime>, loaded: LoadedDatabase) {
    let conf_ready = loaded.config.is_some();
    let conf = app.state::<Config>().clone_();
    *conf.write().await = loaded.config.unwrap_or_default();
    app.set_router(crate::router::app(conf, loaded.db, app.clone()))
        .await;
    let context = app.state::<BootloaderContex>();
    let mut context = context.write().await;
    context.conf_ready = conf_ready;
    context.db_ready = loaded.ready;
}

pub async fn reload(
    app: &tauri::AppHandle<impl Runtime>,
    db: DatabaseConnection,
) -> Result<(), Error> {
    apply_loaded(app, load_database(db).await?).await;
    Ok(())
}

pub async fn snapshot(db: &DatabaseConnection, dst: &Path) -> Result<(), Error> {
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "VACUUM INTO ?",
        [dst.to_string_lossy().to_string().into()],
    ))
    .await
    .map_err(Error::DbConnect)?;
    Ok(())
}

pub async fn backup(db: &DatabaseConnection, data_dir: PathBuf, dst: PathBuf) -> Result<(), Error> {
    let snapshot_path = data_dir.join(format!(".snapshot-{:032x}.db", rand::random::<u128>()));
    let result = async {
        snapshot(db, &snapshot_path).await?;
        let source = snapshot_path.clone();
        tokio::task::spawn_blocking(move || {
            let mut output = std::fs::File::create_new(&dst).map_err(Error::CreateFile)?;
            let result = (|| {
                let mut input = std::fs::File::open(source).map_err(Error::BackupFile)?;
                std::io::copy(&mut input, &mut output).map_err(Error::BackupFile)?;
                output.sync_all().map_err(Error::BackupFile)?;
                Ok::<_, Error>(())
            })();
            drop(output);
            if result.is_err() {
                let _ = std::fs::remove_file(dst);
            }
            result
        })
        .await
        .map_err(task_error)?
    }
    .await;
    let _ = tokio::fs::remove_file(snapshot_path).await;
    result
}

pub async fn restore(
    app: &tauri::AppHandle<impl Runtime>,
    db: &DatabaseConnection,
    source: PathBuf,
    data_dir: PathBuf,
) -> Result<(), Error> {
    let staged = data_dir.join(format!(".restore-{:032x}.db", rand::random::<u128>()));
    let result = async {
        stage_backup(source, staged.clone()).await?;
        validate(&staged).await?;
        let live = data_dir.join(DB_FILE_NAME);
        let previous = data_dir.join(format!("{DB_FILE_NAME}.previous"));
        let (loaded, error) = replace_database(db, &staged, &live, &previous).await?;
        apply_loaded(app, loaded).await;
        error.map_or(Ok(()), Err)
    }
    .await;
    let _ = tokio::fs::remove_file(staged).await;
    result
}

async fn stage_backup(source: PathBuf, target: PathBuf) -> Result<(), Error> {
    tokio::task::spawn_blocking(move || {
        let mut input = std::fs::File::open(source).map_err(Error::OpenBackupFile)?;
        let mut output = std::fs::File::create_new(target).map_err(Error::RestoreBackupFile)?;
        std::io::copy(&mut input, &mut output).map_err(Error::RestoreBackupFile)?;
        output.sync_all().map_err(Error::RestoreBackupFile)?;
        Ok::<_, Error>(())
    })
    .await
    .map_err(task_error)?
}

async fn replace_database(
    db: &DatabaseConnection,
    staged: &Path,
    live: &Path,
    previous: &Path,
) -> Result<(LoadedDatabase, Option<Error>), Error> {
    if previous.exists() {
        tokio::fs::remove_file(previous)
            .await
            .map_err(Error::RestoreBackupFile)?;
    }
    db.close_by_ref().await.map_err(Error::CloseDbError)?;
    let replacement = async {
        tokio::fs::rename(live, previous)
            .await
            .map_err(Error::RestoreBackupFile)?;
        tokio::fs::rename(staged, live)
            .await
            .map_err(Error::RestoreBackupFile)?;
        let restored = connect(live.to_path_buf())
            .await
            .map_err(Error::DbConnect)?;
        match load_database(restored.clone()).await {
            Ok(loaded) => Ok(loaded),
            Err(error) => {
                restored.close().await.map_err(Error::CloseDbError)?;
                Err(error)
            }
        }
    }
    .await;
    match replacement {
        Ok(loaded) => Ok((loaded, None)),
        Err(error) => {
            if previous.exists() {
                if live.exists() {
                    tokio::fs::remove_file(live)
                        .await
                        .map_err(Error::RestoreBackupFile)?;
                }
                tokio::fs::rename(previous, live)
                    .await
                    .map_err(Error::RestoreBackupFile)?;
            }
            let original = connect(live.to_path_buf())
                .await
                .map_err(Error::DbConnect)?;
            Ok((load_database(original).await?, Some(error)))
        }
    }
}

async fn validate(path: &Path) -> Result<(), Error> {
    use tokio::io::AsyncReadExt;
    let mut input = tokio::fs::File::open(path)
        .await
        .map_err(Error::OpenBackupFile)?;
    let mut header = [0; 16];
    input
        .read_exact(&mut header)
        .await
        .map_err(Error::OpenBackupFile)?;
    drop(input);
    if &header != b"SQLite format 3\0" {
        return Err(Error::InvalidBackupFile("不是有效的 SQLite 数据库".into()));
    }
    let db = connect(path.to_path_buf())
        .await
        .map_err(Error::DbConnect)?;
    let result = async {
        let row = db
            .query_one_raw(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA integrity_check",
            ))
            .await
            .map_err(Error::DbConnect)?
            .ok_or_else(|| Error::InvalidBackupFile("数据库完整性检查无结果".into()))?;
        let integrity: String = row
            .try_get("", "integrity_check")
            .map_err(Error::DbConnect)?;
        if integrity != "ok" {
            return Err(Error::InvalidBackupFile(integrity));
        }

        for table in ["student", "item", "record", "seaql_migrations"] {
            let row = db
                .query_one_raw(Statement::from_sql_and_values(
                    DbBackend::Sqlite,
                    "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?",
                    [table.into()],
                ))
                .await
                .map_err(Error::DbConnect)?;
            if row.is_none() {
                return Err(Error::InvalidBackupFile(format!("缺少业务表 {table}")));
            }
        }
        crate::Migrator::up(&db, None)
            .await
            .map_err(Error::DbConnect)?;
        ConfigInner::load(&db).await.map_err(Error::DbConnect)?;
        let violations = db
            .query_all_raw(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_check",
            ))
            .await
            .map_err(Error::DbConnect)?;
        if !violations.is_empty() {
            return Err(Error::InvalidBackupFile("数据库关联数据不完整".into()));
        }
        Ok(())
    }
    .await;
    db.close().await.map_err(Error::CloseDbError)?;
    result
}
