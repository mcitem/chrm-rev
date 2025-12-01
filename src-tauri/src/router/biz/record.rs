use std::collections::HashSet;

use crate::router::{prelude::*, response::error::biz::ExportRecordErr};

use encoding_rs::GBK;
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

async fn export_record(
    State(AppState { ref db, config }): State<AppState>,
    Json(ExportRequest { sign }): Json<ExportRequest>,
) -> Result<AppResponse<serde_json::Value>, ExportRecordErr> {
    let legacy = config.legacy_export_format().await;
    let path = config.export_path().await;

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .await?;

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
                    "{},'{}',{},{},{},{},{},{},{},{}\r\n",
                    count,
                    r.student_no,
                    r.stu_d_level,
                    r.item_id,
                    r.item_name,
                    r.item_spec,
                    r.original_price,
                    r.discount_price,
                    r.quantity,
                    item_price
                );
                total_discount_price += item_price;
            });

            out += &format!(",,,,,,,,,{}\r\n", total_discount_price);
            out += &format!("{},,,,,,,,,\r\n", sign);
            out += "$$$$,,,,,,,,,\r\n";
        }
        true => {
            rec.iter().for_each(|r| {
                for _ in 0..r.quantity {
                    count += 1;
                    out += &format!(
                        "{},'{}',{},{},{},{}\r\n",
                        count,
                        r.student_no,
                        r.stu_d_level,
                        r.item_id,
                        r.discount_price,
                        r.original_price
                    );
                    total_discount_price += r.discount_price;
                }
            });

            out += &format!(",,,,{},\r\n", total_discount_price);
            out += &format!("{},,,,,\r\n", sign);
            out += "$$$$,,,,,\r\n";
        }
    };

    let (bytes, _, _) = GBK.encode(&out);
    file.write_all(&bytes).await?;

    ok!(())
}
