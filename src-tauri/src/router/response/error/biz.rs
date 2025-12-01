use crate::{impl_app_error, router::prelude::AppErr};

use axum::{http::StatusCode, response::IntoResponse};
use sea_orm::ColumnFromStrErr;

#[derive(Debug, derive_more::From)]
pub enum BizErrKind {
    ColumnFromStrErr(ColumnFromStrErr),
    DbErr(sea_orm::DbErr),
    StuNotFound,
    ItemNotFound,
    RecordNotFound,
    ExportRecord(ExportRecordErr),
}

impl_app_error!(
    for BizErrKind;
    INTERNAL_SERVER_ERROR => [
        DbErr(e); format!("数据库错误: {}", e),
        ColumnFromStrErr(e); format!("传入的列名不是有效值{}", e),
        StuNotFound; format!("该学生数据不存在"),
        ItemNotFound; format!("该物品数据不存在"),
        RecordNotFound; format!("该记录不存在"),
    ],
    @delegates;
    ExportRecord,
);

impl IntoResponse for BizErrKind {
    fn into_response(self) -> axum::response::Response {
        let app = AppErr::from(self);
        app.into_response()
    }
}

#[derive(Debug, derive_more::From)]
pub enum ExportRecordErr {
    FileOpenErr(std::io::Error),
    DbErr(sea_orm::DbErr),
}

impl_app_error!(
    for ExportRecordErr;
    INTERNAL_SERVER_ERROR => [
        FileOpenErr(e); format!("打开文件失败: {}", e),
        DbErr(e); format!("数据库错误: {}", e),
    ],
);

impl IntoResponse for ExportRecordErr {
    fn into_response(self) -> axum::response::Response {
        AppErr::BizErr(BizErrKind::ExportRecord(self)).into_response()
    }
}
