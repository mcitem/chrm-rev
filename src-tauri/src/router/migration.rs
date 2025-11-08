use std::path::PathBuf;

use axum::Json;
use calamine::Reader;
use sea_orm::{DatabaseConnection, DbConn, TransactionTrait, TryIntoModel};
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::prelude::*;
use layer::MigrateUpLayer;
mod layer;
pub(super) fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/status", get(status))
        .route("/down", get(down))
        .route("/up", post(up).route_layer(MigrateUpLayer(state)))
}

/// 等同删除所有表（风险接口，请谨慎使用）
async fn down(State(db): State<DatabaseConnection>) -> R {
    let conn = db.begin().await?;
    Migrator::down(&conn, None).await?;
    conn.commit().await?;
    ok!(())
}

use crate::{migration::Migrator, router::response::AppErrExt};
async fn status(State(db): State<DatabaseConnection>) -> R {
    let pending = Migrator::get_pending_migrations(&db).await?;
    ok!(pending.is_empty())
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export, export_to = "./mutation.ts")]
struct MigrateUp {
    #[ts(optional)]
    path: Option<PathBuf>,
    atype: MigrateAtype,
}

#[derive(Deserialize, Serialize, TS, PartialEq, Debug)]
#[ts(export, export_to = "./mutation.ts")]
enum MigrateAtype {
    AnalyzeOnly,
    Import,
    Skip,
}

use super::utils::import::*;
async fn up(State(db): State<DbConn>, Json(MigrateUp { path, atype }): Json<MigrateUp>) -> R {
    // println!("migrate up: {atype:?}, path: {path:?}");
    if atype == MigrateAtype::Skip {
        let conn = db.begin().await?;
        Migrator::up(&conn, None).await?;
        conn.commit().await?;
        return ok!(());
    };

    let path = path.ok_or("缺少文件路径".e_400())?;
    if !(path.is_absolute()
        && path.exists()
        && path.is_file()
        && path.extension().is_some_and(|s| s == "xlsx" || s == "xls"))
    {
        return "仅支持 .xlsx/.xls 文件".e_400().map();
    }

    let mut sheets = calamine::open_workbook_auto(path)?;

    let item_sheet = sheets.worksheet_range("item")?;
    let student_sheet = sheets.worksheet_range("stu")?;

    let mut i_rows = item_sheet.rows().peekable();
    let mut s_rows = student_sheet.rows().peekable();

    //消耗掉标题数据
    let i_title = i_rows.next().ok_or("找不到商品数据标题行".e_400())?;
    let s_title = s_rows.next().ok_or("找不到学生数据标题行".e_400())?;

    let i_ctx = i_analyze(i_title)?;
    let s_ctx = s_analyze(s_title)?;

    if atype == MigrateAtype::AnalyzeOnly {
        let i_first = i_rows
            .peek()
            .map(|v| i_build_active_model(v, &i_ctx, 0, Set(-1)))
            .transpose()?
            .map(|m| m.try_into_model())
            .transpose()?;
        let s_first = s_rows
            .peek()
            .map(|v| s_build_active_model(v, &s_ctx, 0, Set(-1)))
            .transpose()?
            .map(|m| m.try_into_model())
            .transpose()?;
        return ok!({
            "item": i_first,
            "student": s_first,
        });
    }

    if atype == MigrateAtype::Import {
        let conn = db.begin().await?;
        Migrator::up(&conn, None).await?;
        let i_count = i_import(&conn, &i_ctx, i_rows).await?;
        let s_count = s_import(&conn, &s_ctx, s_rows).await?;
        conn.commit().await?;
        return ok!({
            "i_count": i_count,
            "s_count": s_count,
        });
    }

    unreachable!()
}
