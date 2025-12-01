use std::{num::TryFromIntError, path::PathBuf};

use crate::{impl_app_error, router::prelude::AppErr};
use axum::{http::StatusCode, response::IntoResponse};
use derive_more::From;
use sea_orm::DbErr;

#[derive(Debug, From)]
pub enum SystemErrKind {
    GetDataDir(tauri::Error),
    OpenCsvFailed(std::io::Error),
    OpenGitRepoFailed(std::io::Error),
    OpenDataDirFailed(std::io::Error),
    UnsafeInvokeRejected,
    PutConfig(Box<dyn std::error::Error>),
    OpenDocs(tauri::Error, &'static str),
    OpenTimedateCpl(std::io::Error),
    MainNotFound,
    #[from]
    Dataloader(DataloaderErrKind),
    #[from]
    BackupManager(BackupManagerErrKind),
    #[from]
    UnSafe(UnSafeErrKind),
    #[from]
    DataManager(DataManagerErr),
}

impl IntoResponse for SystemErrKind {
    fn into_response(self) -> axum::response::Response {
        AppErr::System(self).into_response()
    }
}

impl_app_error!(
    for SystemErrKind;
    INTERNAL_SERVER_ERROR => [
        OpenCsvFailed(e); format!("打开 CSV 文件失败: {}", e),
        GetDataDir(e); format!("获取数据目录失败: {}", e),
        OpenDataDirFailed(e); format!("打开数据目录失败: {}", e),
        UnsafeInvokeRejected; "不安全的操作被拒绝",
        PutConfig(e); format!("保存配置失败: {}", e),
        OpenDocs(e, msg); format!("打开文档窗口失败[{}]: {}", msg, e),
        OpenTimedateCpl(e); format!("打开时间日期设置窗口失败: {}", e),
        OpenGitRepoFailed(e); format!("打开 Git 仓库失败: {}", e),
        MainNotFound; "主窗口不存在",
    ],
    @delegates;
    Dataloader,
    BackupManager,
    UnSafe,
    DataManager,
);

#[derive(Debug)]
pub enum DataloaderErrKind {
    GetDataDir(tauri::Error),
    AlreadyExists,
    AlreadyConnected,
    DbConnect(DbErr),
    DbMigrate(DbErr),
    CheckDb(DbErr, &'static str),
    DbDirty,
    //
    FileNotExecl(PathBuf),
    SheetNotFound(String),
    DataNotFound(String),
    Calamine(calamine::Error),
    //
    LoadDataDb(DbErr),
    LoadTryFromInt(TryFromIntError),

    FieldNotFound(&'static str, usize, usize),
    Decimal(&'static str, usize, usize, rust_decimal::Error),
}

impl_app_error!(
    for DataloaderErrKind;
    INTERNAL_SERVER_ERROR => [
        GetDataDir(e); format!("获取数据目录失败: {}", e),
        AlreadyExists; "数据库已存在，如需重新初始化，请先删除数据库文件",
        AlreadyConnected; "数据库已连接，如需重新初始化，请先停止数据库连接",
        DbConnect(e); format!("连接数据库失败: {}", e),
        DbMigrate(e); format!("数据库初始化失败: {}", e),
        CheckDb(e, msg); format!("检查数据库失败[{}]: {}", msg, e),
        DbDirty; "当前数据库中已有数据，继续导入会破坏现有数据，请先备份并删除现有数据库",
        FileNotExecl(path); format!("文件不是有效的 Excel 文件: {:?}", path),
        SheetNotFound(sheet); format!("工作表未找到: {}", sheet),
        DataNotFound(data); format!("导入时未找到 {} 数据", data),
        Calamine(e); format!("读取 Excel 文件失败: {}", e),

        LoadDataDb(e); format!("导入数据到数据库失败: {}", e),
        LoadTryFromInt(e); format!("数据量超过限制: {}", e),
        FieldNotFound(s,x,y); format!("导入 {} 时第 {} 行，第{}列为空数据", s, y, x),
        Decimal(s,x,y,e); format!("导入 {} 时第 {} 行，第 {} 列的数值格式错误: {}", s, y, x, e),
    ],
);

#[derive(Debug)]
pub enum BackupManagerErrKind {
    GetDataDir(tauri::Error),
    GetDesktopDir(tauri::Error),
    CreateFile(std::io::Error),
    WalkDir(walkdir::Error),
    StripPrefixError(std::path::StripPrefixError),
    NotUtf8Path(PathBuf),
    BackupFile(std::io::Error),
    ZipError(zip::result::ZipError),
    DbfileNotFound,
    CloseDbError(DbErr),
    DbConnect(DbErr),
    OpenBackupFile(std::io::Error),
    ReadBackupFile(zip::result::ZipError),
    ExtractBackupFile(std::io::Error),
    InvalidBackupFile(String),
    ReadConfigFile(tokio::io::Error),
    DeserializeConfigFile(serde_json::Error),
}

impl_app_error!(
    for BackupManagerErrKind;
    INTERNAL_SERVER_ERROR => [
        GetDataDir(e); format!("获取数据目录失败: {}", e),
        GetDesktopDir(e); format!("获取桌面目录失败: {}", e),
        CreateFile(e); format!("创建备份文件失败: {}", e),
        WalkDir(e); format!("遍历目录失败: {}", e),
        StripPrefixError(e); format!("剥离路径前缀失败: {}", e),
        NotUtf8Path(path); format!("路径不是有效的 UTF-8 字符串: {:?}", path),
        BackupFile(e); format!("备份文件失败: {}", e),
        ZipError(e); format!("创建 ZIP 文件失败: {}", e),
        DbfileNotFound; "数据库文件不存在",
        CloseDbError(e); format!("关闭数据库连接失败: {}", e),
        DbConnect(e); format!("连接数据库失败: {}", e),
        OpenBackupFile(e); format!("打开备份文件失败: {}", e),
        ReadBackupFile(e); format!("读取备份文件失败: {}", e),
        ExtractBackupFile(e); format!("解压备份文件失败: {}", e),
        InvalidBackupFile(msg); format!("备份文件无效: {}", msg),
        ReadConfigFile(e); format!("读取配置文件失败: {}", e),
        DeserializeConfigFile(e); format!("解析配置文件失败: {}", e),
    ],
);

#[derive(Debug)]
pub enum UnSafeErrKind {
    CloseDbError(DbErr),
    ClearDbFileFailed(std::io::Error),
    ClearRecordFailed(DbErr),
    ClearDataDirFailed(std::io::Error),
    ReCreateDataDirFailed(std::io::Error),
    GetDataDir(tauri::Error),
}

impl_app_error!(
    for UnSafeErrKind;
    INTERNAL_SERVER_ERROR => [
        CloseDbError(e); format!("关闭数据库连接失败: {}", e),
        ClearDbFileFailed(e); format!("清除数据库文件失败: {}", e),
        ClearRecordFailed(e); format!("清除记录失败: {}", e),
        ClearDataDirFailed(e); format!("清除数据目录失败: {}", e),
        ReCreateDataDirFailed(e); format!("重新创建数据目录失败: {}", e),
        GetDataDir(e); format!("获取数据目录失败: {}", e),

    ],
);

#[derive(Debug, From)]
pub enum DataManagerErr {
    Tauri(tauri::Error),
    DbErr(DbErr),
    XlsxErr(rust_xlsxwriter::XlsxError),
    DataInConsistentWraning,
}

impl_app_error!(
    for DataManagerErr;
    INTERNAL_SERVER_ERROR => [
        Tauri(e); format!("Tauri 错误: {}", e),
        DbErr(e); format!("数据库错误: {}", e),
        XlsxErr(e); format!("Excel 处理错误: {}", e),
        DataInConsistentWraning; None,
    ],
);

impl IntoResponse for DataManagerErr {
    fn into_response(self) -> axum::response::Response {
        AppErr::System(SystemErrKind::DataManager(self)).into_response()
    }
}
