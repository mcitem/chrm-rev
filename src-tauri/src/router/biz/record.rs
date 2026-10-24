use std::collections::HashSet;

use crate::router::{prelude::*, response::error::biz::ExportRecordErr};

use encoding_rs::GBK;
use sea_orm::TransactionTrait;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/all", get(get_all_record))
        .route("/all_with_summary", get(get_all_record_with_summary))
        .route("/export", post(export_record))
}

type R = Result<AppResponse<serde_json::Value>, BizErrKind>;

#[derive(Serialize, TS)]
#[ts(export, export_to = "summary.ts")]
struct Summary {
    stu_count: usize,
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    total_discount_price: Decimal,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "summary.ts")]
struct AllRecordWithSummary {
    records: Vec<record::Model>,
    summary: Summary,
}

async fn get_all_record_with_summary(State(ref db): State<DbConn>) -> R {
    let r = record::Entity::find().all(db).await?;

    let stu_count = r.iter().map(|r| r.student_id).collect::<HashSet<_>>().len();

    let total_discount_price = r
        .iter()
        .map(|r| r.discount_price * Decimal::from(r.quantity))
        .sum::<Decimal>();

    ok!(AllRecordWithSummary {
        records: r,
        summary: Summary {
            stu_count,
            total_discount_price,
        },
    })
}

async fn get_all_record(State(ref db): State<DbConn>) -> R {
    let r = record::Entity::find().all(db).await?;
    ok!(r)
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "mutation.ts")]
struct ExportRequest {
    pub sign: String,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "mutation.ts")]
struct ExportResponse {
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    total_discount_price: Decimal,
    out: String,
}

async fn export_record(
    State(AppState { ref db, config }): State<AppState>,
    Json(ExportRequest { sign }): Json<ExportRequest>,
) -> Result<AppResponse<serde_json::Value>, ExportRecordErr> {
    let _operation = config.operation().await;
    let legacy = config.legacy_export_format().await;
    let path = std::path::absolute(config.export_path().await?)?;

    let rec = record::Entity::find().all(db).await?;

    let mut count = 0;
    let mut total_discount_price = Decimal::new(0, 2);

    let mut out = String::new();

    match legacy {
        false => {
            rec.iter().for_each(|r| {
                count += 1;
                let item_price = r.discount_price * Decimal::from(r.quantity);
                out += &format!(
                    "{},{},{},{},{},{},{},{},{},{}\r\n",
                    count,
                    csv_field(&format!("'{}'", r.student_no)),
                    r.stu_d_level,
                    r.item_id,
                    csv_field(&r.item_name),
                    csv_field(&r.item_spec),
                    r.original_price,
                    r.discount_price,
                    r.quantity,
                    item_price
                );
                total_discount_price += item_price;
            });

            out += &format!(",,,,,,,,,{}\r\n", total_discount_price);
            out += &format!("{},,,,,,,,,\r\n", csv_field(&sign));
            out += "$$$$,,,,,,,,,\r\n";
        }
        true => {
            rec.iter().for_each(|r| {
                for _ in 0..r.quantity {
                    count += 1;
                    out += &format!(
                        "{},{},{},{},{},{}\r\n",
                        count,
                        csv_field(&format!("'{}'", r.student_no)),
                        r.stu_d_level,
                        r.item_id,
                        r.discount_price,
                        r.original_price
                    );
                    total_discount_price += r.discount_price;
                }
            });

            out += &format!(",,,,{},\r\n", total_discount_price);
            out += &format!("{},,,,,\r\n", csv_field(&sign));
            out += "$$$$,,,,,\r\n";
        }
    };

    let response = ExportResponse {
        total_discount_price,
        out,
    };
    if rec.is_empty() {
        return ok!(response);
    }
    let (bytes, _, had_errors) = GBK.encode(&response.out);
    if had_errors {
        return Err(
            std::io::Error::other("导出内容含 GBK 无法表示的字符，请修改签名或商品信息").into(),
        );
    }
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .await?;
    file.write_all(&bytes).await?;
    file.sync_all().await?;

    let txn = db.begin().await?;
    let ids: Vec<_> = rec.iter().map(|r| r.id).collect();
    // Keep each query within SQLite's parameter limit.
    for chunk in ids.chunks(900) {
        record::Entity::delete_many()
            .filter(record::Column::Id.is_in(chunk.iter().copied()))
            .exec(&txn)
            .await?;
    }
    txn.commit().await?;
    ok!(response)
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}
