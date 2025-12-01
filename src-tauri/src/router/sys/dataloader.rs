use super::super::prelude::*;
use crate::{
    config::DB_FILE_NAME,
    entity::{item, student},
    router::{response::error::system::DataloaderErrKind, sys::dataloader::common::*},
    utils::{NextPrimaryKey, SqlitePath},
};
use calamine::Reader;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, Insert, TransactionTrait};
use sea_orm_migration::MigratorTrait;
use tauri_plugin_axum::AxumExt;
use ts_rs::TS;

pub mod common;

pub fn router<RT: Runtime>() -> Router<ComplexState<RT>> {
    Router::new()
        .route("/skip", post(set_db_ready))
        .route("/init", post(init_db).put(idem_init_db))
        .route("/finish", post(set_db_ready))
        .route("/check", post(check_db))
        .route("/load/worksheet", post(load_worksheet))
        .route("/load/worksheet/data", post(load_worksheet_data))
        .route("/load/item/check", post(load_item_check))
        .route("/load/item", post(load_item))
        .route("/load/stu/check", post(load_stu_check))
        .route("/load/stu", post(load_stu))
        .route("/next_pk", get(get_next_px))
}

async fn set_db_ready(State(ComplexState { app, .. }): State<ComplexState<impl Runtime>>) -> R {
    let conf = app.state::<Config>().clone_();

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(DataloaderErrKind::GetDataDir)?;

    let db_path = data_dir.join(DB_FILE_NAME);

    let db_path: ConnectOptions = SqlitePath(db_path).into();

    let db_path = Database::connect(db_path)
        .await
        .map_err(DataloaderErrKind::DbConnect)?;

    db_path.ping().await.map_err(DataloaderErrKind::DbConnect)?;

    let pending = crate::Migrator::get_pending_migrations(&db_path)
        .await
        .map_err(DataloaderErrKind::DbMigrate)?;

    if !pending.is_empty() {
        crate::Migrator::up(&db_path, None)
            .await
            .map_err(DataloaderErrKind::DbMigrate)?;
    }

    app.set_router(crate::router::app(conf, db_path, app.clone()))
        .await;

    {
        let ctx = app.state::<BootloaderContex>();
        ctx.write().await.db_ready = true;
    }

    ok!(())
}

async fn load_stu(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
    Json(CheckStuImport {
        path,
        sheet,
        ctx,
        data_include_header,
    }): Json<CheckStuImport>,
) -> R {
    let sheet = use_range_data(path, sheet)?;

    let data_include_header = data_include_header as usize;

    let default_balance = app.state::<Config>().balance_config().await.default_balance;

    let items = sheet
        .rows()
        .skip(data_include_header)
        .enumerate()
        .map(|(y, row)| {
            Ok(s_build_active_model(
                y,
                row,
                data_include_header,
                &ctx,
                default_balance,
            )?)
        })
        .collect::<Result<Vec<_>, AppErr>>()?;

    let len = items.len();

    let txn = db.begin().await.map_err(DataloaderErrKind::LoadDataDb)?;

    Insert::many(items)
        .exec(&txn)
        .await
        .map_err(DataloaderErrKind::LoadDataDb)?;

    txn.commit().await.map_err(DataloaderErrKind::LoadDataDb)?;

    ok!(len)
}

async fn load_stu_check(
    Json(CheckStuImport {
        path,
        sheet,
        ctx,
        data_include_header,
    }): Json<CheckStuImport>,
) -> R {
    let sheet = use_range_data(path, sheet)?;

    let data_include_header = data_include_header as usize;

    let total = sheet.rows().len() - data_include_header;

    let first_data = sheet
        .rows()
        .skip(data_include_header)
        .next()
        .ok_or(DataloaderErrKind::DataNotFound(String::from("第一行商品")))?;

    let first_item = TryLoadedStuModel {
        id: ctx
            .id
            .and_then(|idx| first_data.get(idx).map(|c| c.to_string())),
        no: first_data.get(ctx.s_no).map(|c| c.to_string()),
        name: first_data.get(ctx.s_name).map(|c| c.to_string()),
        d_level: first_data.get(ctx.s_d_level).map(|c| c.to_string()),
        second_school: ctx
            .s_second_school
            .and_then(|idx| first_data.get(idx).map(|c| c.to_string())),
        class: ctx
            .s_class
            .and_then(|idx| first_data.get(idx).map(|c| c.to_string())),
        credit: ctx
            .s_credit
            .and_then(|idx| first_data.get(idx).map(|c| c.to_string())),
        sex: ctx
            .s_sex
            .and_then(|idx| first_data.get(idx).map(|c| c.to_string())),
        major: ctx
            .s_major
            .and_then(|idx| first_data.get(idx).map(|c| c.to_string())),
    };

    ok!(CheckStuImportReturn {
        first_item,
        ctx,
        total
    })
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
pub struct GetNextPk {
    item: i32,
    stu: i32,
}

impl GetNextPk {
    pub async fn new(db: &DatabaseConnection) -> Result<GetNextPk, DataloaderErrKind> {
        let item = item::Entity::next_pk(db)
            .await
            .map_err(|e| DataloaderErrKind::CheckDb(e, "item"))?;

        let stu = student::Entity::next_pk(db)
            .await
            .map_err(|e| DataloaderErrKind::CheckDb(e, "student"))?;

        Ok(GetNextPk { item, stu })
    }
}

async fn get_next_px(State(ComplexState { ref db, .. }): State<ComplexState<impl Runtime>>) -> R {
    ok!(GetNextPk::new(db).await?)
}

async fn load_item(
    State(ComplexState { db, .. }): State<ComplexState<impl Runtime>>,
    Json(CheckItemImport {
        path,
        sheet,
        ctx,
        data_include_header,
    }): Json<CheckItemImport>,
) -> R {
    let sheet = use_range_data(path, sheet)?;

    let data_include_header = data_include_header as usize;

    let items = sheet
        .rows()
        .skip(data_include_header)
        .enumerate()
        .map(|(y, row)| Ok(i_build_active_model(y, row, data_include_header, &ctx)?))
        .collect::<Result<Vec<_>, AppErr>>()?;

    let len = items.len();

    let txn = db.begin().await.map_err(DataloaderErrKind::LoadDataDb)?;

    Insert::many(items)
        .exec(&txn)
        .await
        .map_err(DataloaderErrKind::LoadDataDb)?;

    txn.commit().await.map_err(DataloaderErrKind::LoadDataDb)?;

    ok!(len)
}

/// 索引从0开始
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
pub struct LoadItemContext {
    id: Option<usize>,
    /// 商品名称
    i_name: usize,
    /// 规格
    i_spec: usize,
    /// 原价
    i_p: usize,
    /// 3折价
    i_p_hard: usize,
    /// 5折价
    i_p_easy: usize,
    /// 7折价
    i_p_normal: usize,
    /// 积分
    i_p_score: usize,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
pub struct LoadStuContext {
    id: Option<usize>,
    /// 学生姓名
    s_name: usize,
    /// 学号
    s_no: usize,
    /// 认定级别
    s_d_level: usize,
    // 学院
    s_second_school: Option<usize>,
    // 班级
    s_class: Option<usize>,
    // 性别
    s_sex: Option<usize>,
    // 余额
    s_credit: Option<usize>,
    // 专业
    s_major: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct TryLoadedItemModel {
    id: Option<String>,
    name: Option<String>,
    spec: Option<String>,
    p: Option<String>,
    p_hard: Option<String>,
    p_easy: Option<String>,
    p_normal: Option<String>,
    p_score: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct TryLoadedStuModel {
    id: Option<String>,
    name: Option<String>,
    no: Option<String>,
    d_level: Option<String>,
    second_school: Option<String>,
    class: Option<String>,
    sex: Option<String>,
    credit: Option<String>,
    major: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct CheckItemImport {
    path: String,
    sheet: String,
    data_include_header: bool,
    ctx: LoadItemContext,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct CheckStuImport {
    path: String,
    sheet: String,
    data_include_header: bool,
    ctx: LoadStuContext,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct CheckItemImportReturn {
    first_item: TryLoadedItemModel,
    total: usize,
    ctx: LoadItemContext,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct CheckStuImportReturn {
    first_item: TryLoadedStuModel,
    total: usize,
    ctx: LoadStuContext,
}

// 返回匹配到的数据
async fn load_item_check(
    Json(CheckItemImport {
        path,
        sheet,
        ctx,
        data_include_header,
    }): Json<CheckItemImport>,
) -> R {
    let sheet = use_range_data(path, sheet)?;

    let data_include_header = data_include_header as usize;

    let total = sheet.rows().len() - data_include_header;

    let first_data = sheet
        .rows()
        .skip(data_include_header)
        .next()
        .ok_or(DataloaderErrKind::DataNotFound(String::from("第一行学生")))?;

    let first_item = TryLoadedItemModel {
        id: ctx
            .id
            .and_then(|idx| first_data.get(idx).map(|c| c.to_string())),
        name: first_data.get(ctx.i_name).map(|c| c.to_string()),
        spec: first_data.get(ctx.i_spec).map(|c| c.to_string()),
        p: first_data.get(ctx.i_p).map(|c| c.to_string()),
        p_hard: first_data.get(ctx.i_p_hard).map(|c| c.to_string()),
        p_easy: first_data.get(ctx.i_p_easy).map(|c| c.to_string()),
        p_normal: first_data.get(ctx.i_p_normal).map(|c| c.to_string()),
        p_score: first_data.get(ctx.i_p_score).map(|c| c.to_string()),
    };

    ok!(CheckItemImportReturn {
        first_item,
        ctx,
        total
    })
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct LoadWorksheetData {
    path: String,
    sheet: String,
}

/// 只返回前五行数据
async fn load_worksheet_data(
    Json(LoadWorksheetData { path, sheet }): Json<LoadWorksheetData>,
) -> R {
    let sheet = use_range_data(path, sheet)?;
    let data = sheet
        .rows()
        .take(5)
        .map(|row| row.iter().map(ToString::to_string).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    ok!(data)
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct LoadWorksheet {
    path: String,
}

/// 获得工作表名
async fn load_worksheet(Json(LoadWorksheet { path }): Json<LoadWorksheet>) -> R {
    let path = check_path(path)?;
    let sheets = use_sheets(path)?;
    let sheet_names = sheets.sheet_names();
    ok!(sheet_names)
}

async fn check_db(State(ComplexState { ref db, .. }): State<ComplexState<impl Runtime>>) -> R {
    let GetNextPk { item, stu } = GetNextPk::new(db).await?;

    tracing::debug!("check db: item next pk: {}, student next pk: {}", item, stu);

    if stu != 1 || item != 1 {
        return Err(DataloaderErrKind::DbDirty.into());
    }

    ok!(())
}

async fn init_db(State(ComplexState { app, db }): State<ComplexState<impl Runtime>>) -> R {
    if let Ok(()) = db.ping().await {
        return Err(DataloaderErrKind::AlreadyConnected.into());
    }

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(DataloaderErrKind::GetDataDir)?;

    let db_path = data_dir.join(DB_FILE_NAME);

    if db_path.exists() {
        return Err(DataloaderErrKind::AlreadyExists)?;
    }

    let db_path: ConnectOptions = SqlitePath(db_path).into();

    let db_path = Database::connect(db_path)
        .await
        .map_err(DataloaderErrKind::DbConnect)?;

    db_path.ping().await.map_err(DataloaderErrKind::DbConnect)?;

    crate::Migrator::up(&db_path, None)
        .await
        .map_err(DataloaderErrKind::DbMigrate)?;

    let conf = app.state::<Config>().clone_();

    app.set_router(crate::router::app(conf, db_path, app.clone()))
        .await;

    ok!(())
}

/// 幂等接口，已连接数据库则直接返回成功
async fn idem_init_db(s: State<ComplexState<impl Runtime>>) -> R {
    if let Ok(()) = s.db.ping().await {
        return ok!(());
    }

    init_db(s).await
}
