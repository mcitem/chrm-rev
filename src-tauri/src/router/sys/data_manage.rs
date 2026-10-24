use std::path::PathBuf;

use rand::distr::Alphanumeric;
use rust_xlsxwriter::{workbook::Workbook, worksheet::Worksheet};

use crate::router::{prelude::*, response::error::system::DataManagerErr};

pub fn router<RT: Runtime>() -> Router<ComplexState<RT>> {
    Router::new()
        .route("/export_all", post(export_all))
        .route("/export_all/check", post(export_all_check))
}

type R = Result<AppResponse<serde_json::Value>, DataManagerErr>;

async fn export_all_check(
    State(ComplexState { ref db, .. }): State<ComplexState<impl Runtime>>,
) -> R {
    check_records(db).await?;

    ok!(())
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "./dataManager.ts")]
struct ExportAll {
    path: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "./dataManager.ts")]
struct ExportAllResponse {
    dst: PathBuf,
}

async fn export_all(
    State(ComplexState { ref db, app }): State<ComplexState<impl Runtime>>,
    Json(ExportAll { path }): Json<ExportAll>,
) -> R {
    let path = match path {
        Some(p) => PathBuf::from(p),
        None => {
            use rand::RngExt;
            let random_suffix: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .map(char::from)
                .take(6)
                .collect();

            app.path().desktop_dir()?.join(format!(
                "chrm-rev_{}_{}.xlsx",
                chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"),
                random_suffix,
            ))
        }
    };

    let conf = app.state::<Config>().clone_();
    let _operation = conf.operation().await;
    check_records(db).await?;
    let items = item::Entity::find().all(db).await?;
    let students = student::Entity::find().all(db).await?;
    let output = path.clone();
    tokio::task::spawn_blocking(move || {
        let mut item_sheet = Worksheet::new();
        item_sheet.set_name("item")?;
        item_sheet.deserialize_headers::<item::Model>(0, 0)?;
        item_sheet.serialize(&items)?;
        let mut stu_sheet = Worksheet::new();
        stu_sheet.set_name("stu")?;
        stu_sheet.deserialize_headers::<student::Model>(0, 0)?;
        stu_sheet.serialize(&students)?;
        let mut workbook = Workbook::new();
        workbook.push_worksheet(item_sheet);
        workbook.push_worksheet(stu_sheet);
        workbook.save(output)?;
        Ok::<_, rust_xlsxwriter::XlsxError>(())
    })
    .await
    .map_err(|e| DataManagerErr::DbErr(sea_orm::DbErr::Custom(e.to_string())))??;

    ok!(ExportAllResponse { dst: path })
}

async fn check_records(db: &DbConn) -> Result<(), DataManagerErr> {
    if record::Entity::find().one(db).await?.is_some() {
        return Err(DataManagerErr::DataInConsistentWraning);
    }
    Ok(())
}
