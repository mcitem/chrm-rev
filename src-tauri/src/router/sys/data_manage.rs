use std::path::PathBuf;

use rand::distr::Alphanumeric;
use rust_xlsxwriter::{workbook::Workbook, worksheet::Worksheet};

use crate::{
    router::{prelude::*, response::error::system::DataManagerErr},
    utils::NextPrimaryKey,
};

pub fn router<RT: Runtime>() -> Router<ComplexState<RT>> {
    Router::new()
        .route("/export_all", post(export_all))
        .route("/export_all/check", post(export_all_check))
}

type R = Result<AppResponse<serde_json::Value>, DataManagerErr>;

async fn export_all_check(
    State(ComplexState { ref db, .. }): State<ComplexState<impl Runtime>>,
) -> R {
    let record = record::Entity::next_pk(db).await?;

    tracing::info!("record: {}", record);

    if record != 1 {
        return Err(DataManagerErr::DataInConsistentWraning);
    }

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

    let mut item_sheet = Worksheet::new();

    item_sheet.set_name("item")?;

    let items = item::Entity::find().all(db).await?;

    item_sheet.deserialize_headers::<item::Model>(0, 0)?;

    item_sheet.serialize(&items)?;

    let mut stu_sheet = Worksheet::new();

    stu_sheet.set_name("stu")?;

    let students = student::Entity::find().all(db).await?;

    stu_sheet.deserialize_headers::<student::Model>(0, 0)?;

    stu_sheet.serialize(&students)?;

    let mut workbook = Workbook::new();

    workbook.push_worksheet(item_sheet);
    workbook.push_worksheet(stu_sheet);

    workbook.save(path.clone())?;

    ok!(ExportAllResponse { dst: path })
}
