use std::path::{Path, PathBuf};

use super::super::prelude::*;

#[derive(Deserialize)]
pub(super) struct ExportPath {
    path: PathBuf,
}

pub(super) async fn is_available(path: &Path) -> std::io::Result<bool> {
    if !path.is_absolute()
        || !path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
    {
        return Ok(false);
    }
    match tokio::fs::metadata(path).await {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub(super) async fn save(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
    Json(ExportPath { path }): Json<ExportPath>,
) -> super::R {
    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    if !is_available(&path)
        .await
        .map_err(SystemErrKind::ConfigureExportPath)?
    {
        return Err(SystemErrKind::ConfigureExportPath(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "请选择已有的 CSV 文件或先创建新电子表",
        )));
    }
    let mut current = conf.write().await;
    let mut updated = current.clone();
    updated.export_path = path;
    updated
        .save(&db)
        .await
        .map_err(|error| SystemErrKind::PutConfig(Box::new(error)))?;
    *current = updated;
    ok!(())
}

#[derive(Deserialize)]
pub(super) struct CreateExportFile {
    directory: PathBuf,
}

pub(super) async fn create(Json(request): Json<CreateExportFile>) -> R {
    let stem = format!(
        "电子表_{}",
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
    );
    let path = create_in_directory(&request.directory, &stem)
        .await
        .map_err(SystemErrKind::CreateExportFile)?;
    ok!({ "path": path })
}

async fn create_in_directory(directory: &Path, stem: &str) -> std::io::Result<PathBuf> {
    if !directory.is_absolute() || !tokio::fs::metadata(directory).await?.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "请选择有效的保存文件夹",
        ));
    }
    for suffix in 0..1000 {
        let filename = if suffix == 0 {
            format!("{stem}.csv")
        } else {
            format!("{stem}_{suffix}.csv")
        };
        let path = directory.join(filename);
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(file) => {
                file.sync_all().await?;
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "同名电子表过多，请稍后重试或选择其他文件夹",
    ))
}
