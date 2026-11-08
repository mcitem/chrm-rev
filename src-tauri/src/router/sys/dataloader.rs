use super::super::prelude::*;
use crate::{
    config::DB_FILE_NAME,
    entity::{item, student},
    router::{response::error::system::DataloaderErrKind, sys::dataloader::common::*},
    utils::{NextPrimaryKey, SqlitePath},
};
use calamine::Reader;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tauri_plugin_axum::AxumExt;
use ts_rs::TS;

mod batch;
pub mod common;
use batch::insert_in_batches;

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

async fn set_db_ready(State(ComplexState { app, db }): State<ComplexState<impl Runtime>>) -> R {
    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    crate::utils::set_data_ready(&db, true)
        .await
        .map_err(DataloaderErrKind::DbMigrate)?;
    app.state::<BootloaderContex>().write().await.db_ready = true;
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
    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    let sheet = blocking(move || use_range_data(path, sheet)).await?;

    let data_include_header = data_include_header as usize;
    let first_id = if ctx.id.is_none() {
        student::Entity::next_pk(&db)
            .await
            .map_err(DataloaderErrKind::LoadDataDb)?
    } else {
        1
    };

    let default_balance = app.state::<Config>().balance_config().await.default_balance;

    let items = blocking(move || {
        sheet
            .rows()
            .skip(data_include_header)
            .enumerate()
            .map(|(y, row)| {
                s_build_active_model(y, row, data_include_header, &ctx, default_balance, first_id)
                    .map_err(|error| located(error, sheet.start()))
            })
            .collect::<Result<Vec<_>, DataloaderErrKind>>()
    })
    .await?;

    let len = insert_in_batches(&db, items)
        .await
        .map_err(DataloaderErrKind::LoadDataDb)?;

    ok!(len)
}

async fn load_stu_check(
    State(ComplexState { app, db }): State<ComplexState<impl Runtime>>,
    Json(CheckStuImport {
        path,
        sheet,
        ctx,
        data_include_header,
    }): Json<CheckStuImport>,
) -> R {
    let sheet = blocking(move || use_range_data(path, sheet)).await?;

    let data_include_header = data_include_header as usize;
    let first_id = if ctx.id.is_none() {
        student::Entity::next_pk(&db)
            .await
            .map_err(DataloaderErrKind::LoadDataDb)?
    } else {
        1
    };

    let total = sheet.rows().len().saturating_sub(data_include_header);

    let first_data = sheet
        .rows()
        .skip(data_include_header)
        .next()
        .ok_or(DataloaderErrKind::DataNotFound(String::from("第一行学生")))?;

    let default_balance = app.state::<Config>().default_balance().await;
    let validation_sheet = sheet.clone();
    let validation_ctx = ctx.clone();
    let difficulty_counts = blocking(move || {
        student_import_counts(
            &validation_sheet,
            data_include_header,
            &validation_ctx,
            default_balance,
            first_id,
        )
    })
    .await?;
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
        total,
        difficulty_counts,
    })
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
pub struct GetNextPk {
    item: i32,
    stu: i32,
    #[ts(type = "number")]
    item_count: u64,
    #[ts(type = "number")]
    stu_count: u64,
}

impl GetNextPk {
    pub async fn new(db: &DatabaseConnection) -> Result<GetNextPk, DataloaderErrKind> {
        let item = item::Entity::next_pk(db)
            .await
            .map_err(|e| DataloaderErrKind::CheckDb(e, "item"))?;

        let stu = student::Entity::next_pk(db)
            .await
            .map_err(|e| DataloaderErrKind::CheckDb(e, "student"))?;

        use sea_orm::PaginatorTrait;
        let item_count = item::Entity::find()
            .count(db)
            .await
            .map_err(|e| DataloaderErrKind::CheckDb(e, "item"))?;
        let stu_count = student::Entity::find()
            .count(db)
            .await
            .map_err(|e| DataloaderErrKind::CheckDb(e, "student"))?;
        Ok(GetNextPk {
            item,
            stu,
            item_count,
            stu_count,
        })
    }
}

async fn get_next_px(State(ComplexState { ref db, .. }): State<ComplexState<impl Runtime>>) -> R {
    ok!(GetNextPk::new(db).await?)
}

async fn load_item(
    State(ComplexState { db, app }): State<ComplexState<impl Runtime>>,
    Json(CheckItemImport {
        path,
        sheet,
        ctx,
        data_include_header,
        allow_non_positive,
    }): Json<CheckItemImport>,
) -> R {
    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    let sheet = blocking(move || use_range_data(path, sheet)).await?;

    let data_include_header = data_include_header as usize;
    let first_id = if ctx.id.is_none() {
        item::Entity::next_pk(&db)
            .await
            .map_err(DataloaderErrKind::LoadDataDb)?
    } else {
        1
    };

    let items = blocking(move || {
        let items = sheet
            .rows()
            .skip(data_include_header)
            .enumerate()
            .map(|(y, row)| {
                i_build_active_model(y, row, data_include_header, &ctx, first_id)
                    .map_err(|error| located(error, sheet.start()))
            })
            .collect::<Result<Vec<_>, DataloaderErrKind>>()?;
        let warnings = item_warnings(&sheet, &ctx, data_include_header);
        if !allow_non_positive && !warnings.is_empty() {
            return Err(DataloaderErrKind::NonPositive(
                warnings
                    .iter()
                    .map(|w| format!("第{}行第{}列 {}={}", w.row, w.column, w.field, w.value))
                    .collect::<Vec<_>>()
                    .join("；"),
            ));
        }
        Ok(items)
    })
    .await?;

    let len = insert_in_batches(&db, items)
        .await
        .map_err(DataloaderErrKind::LoadDataDb)?;

    ok!(len)
}

/// 索引从0开始
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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
    #[serde(default)]
    allow_non_positive: bool,
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
    warnings: Vec<ImportWarning>,
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
    difficulty_counts: StudentDifficultyCounts,
}

#[derive(Debug, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct StudentDifficultyCounts {
    peaceful: usize,
    easy: usize,
    normal: usize,
    hard: usize,
}

impl StudentDifficultyCounts {
    fn add(&mut self, difficulty: student::Difficulty) {
        match difficulty {
            student::Difficulty::Peaceful => self.peaceful += 1,
            student::Difficulty::Easy => self.easy += 1,
            student::Difficulty::Normal => self.normal += 1,
            student::Difficulty::Hard => self.hard += 1,
        }
    }
}

fn student_import_counts(
    sheet: &calamine::Range<calamine::Data>,
    header: usize,
    ctx: &LoadStuContext,
    default_balance: crate::config::DefaultBalance,
    first_id: i32,
) -> Result<StudentDifficultyCounts, DataloaderErrKind> {
    let mut counts = StudentDifficultyCounts::default();
    for (y, row) in sheet.rows().skip(header).enumerate() {
        let model = s_build_active_model(y, row, header, ctx, default_balance, first_id)
            .map_err(|error| located(error, sheet.start()))?;
        counts.add(model.difficulty_level.unwrap());
    }
    Ok(counts)
}

// 返回匹配到的数据
async fn load_item_check(
    State(ComplexState { db, .. }): State<ComplexState<impl Runtime>>,
    Json(CheckItemImport {
        path,
        sheet,
        ctx,
        data_include_header,
        allow_non_positive: _,
    }): Json<CheckItemImport>,
) -> R {
    let sheet = blocking(move || use_range_data(path, sheet)).await?;

    let data_include_header = data_include_header as usize;
    let first_id = if ctx.id.is_none() {
        item::Entity::next_pk(&db)
            .await
            .map_err(DataloaderErrKind::LoadDataDb)?
    } else {
        1
    };

    let total = sheet.rows().len().saturating_sub(data_include_header);

    let first_data = sheet
        .rows()
        .skip(data_include_header)
        .next()
        .ok_or(DataloaderErrKind::DataNotFound(String::from("第一行商品")))?;

    // Preview validates every row, not only the displayed first row.
    let validation_sheet = sheet.clone();
    let validation_ctx = ctx.clone();
    let warnings = blocking(move || {
        for (y, row) in validation_sheet
            .rows()
            .skip(data_include_header)
            .enumerate()
        {
            i_build_active_model(y, row, data_include_header, &validation_ctx, first_id)
                .map_err(|error| located(error, validation_sheet.start()))?;
        }
        Ok(item_warnings(
            &validation_sheet,
            &validation_ctx,
            data_include_header,
        ))
    })
    .await?;
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
        warnings,
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
    let sheet = blocking(move || use_range_data(path, sheet)).await?;
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
    let sheet_names = blocking(move || {
        let sheets = use_sheets(check_path(path)?)?;
        Ok(sheets.sheet_names())
    })
    .await?;
    ok!(sheet_names)
}

async fn check_db(State(ComplexState { ref db, .. }): State<ComplexState<impl Runtime>>) -> R {
    if item::Entity::find()
        .one(db)
        .await
        .map_err(|e| DataloaderErrKind::CheckDb(e, "item"))?
        .is_some()
        || student::Entity::find()
            .one(db)
            .await
            .map_err(|e| DataloaderErrKind::CheckDb(e, "student"))?
            .is_some()
    {
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
    let _operation = conf.operation().await;

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

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, DataloaderErrKind> + Send + 'static,
) -> Result<T, DataloaderErrKind> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| DataloaderErrKind::Worker(e.to_string()))?
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./dataloader.ts")]
struct ImportWarning {
    row: usize,
    column: usize,
    field: String,
    value: String,
}

fn item_warnings(
    sheet: &calamine::Range<calamine::Data>,
    ctx: &LoadItemContext,
    header: usize,
) -> Vec<ImportWarning> {
    let mut warnings = Vec::new();
    let (row_offset, column_offset) = sheet.start().unwrap_or_default();
    for (y, row) in sheet.rows().skip(header).enumerate() {
        for (index, field) in [
            (ctx.i_p, "原价"),
            (ctx.i_p_hard, "特别困难价格"),
            (ctx.i_p_normal, "困难价格"),
            (ctx.i_p_easy, "一般困难价格"),
            (ctx.i_p_score, "积分价格"),
        ] {
            let value = row.get(index).map(ToString::to_string).unwrap_or_default();
            if value.parse::<Decimal>().is_ok_and(|v| v <= Decimal::ZERO) {
                warnings.push(ImportWarning {
                    row: y + header + 1 + row_offset as usize,
                    column: index + 1 + column_offset as usize,
                    field: field.into(),
                    value,
                });
            }
        }
    }
    warnings
}
