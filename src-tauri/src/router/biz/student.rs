use std::collections::HashSet;

use sea_orm::{EntityLoaderTrait, ExprTrait, IntoActiveModel, TransactionTrait};

use crate::{
    entity::student::{self, Difficulty},
    router::prelude::*,
};

type R = Result<AppResponse<serde_json::Value>, BizErrKind>;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/{id}", get(get_stu))
        .route("/{id}/record", get(get_stu_record))
        .route(
            "/{id}/record_with_summary",
            get(get_stu_record_with_summary),
        )
        .route(
            "/{stu_id}/record/{item_id}",
            post(set_item).delete(delete_item),
        )
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "mutation.ts")]
struct OperationRecord {
    pub item_id: i32,
    pub stu_id: i32,
}

// 删除学生记录
// 返还余额
async fn delete_item(
    State(AppState { ref db, config }): State<AppState>,
    Path(OperationRecord { stu_id, item_id }): Path<OperationRecord>,
) -> R {
    let ctx = db.begin().await?;

    let conf = config.balance_config().await;

    let mut r = record::Entity::load()
        .filter(record::COLUMN.item_id.eq(item_id))
        .filter(record::COLUMN.student_id.eq(stu_id))
        .with(student::Entity)
        .one(&ctx)
        .await?
        .ok_or(BizErrKind::RecordNotFound)?;

    let stu = r.student.take().ok_or(BizErrKind::StuNotFound)?;

    let new_balance = stu.balance
        + Decimal::from(r.quantity)
            * match conf.pay_for_original_price {
                true => r.original_price,
                false => r.discount_price,
            };

    stu.into_active_model()
        .set_balance(new_balance)
        .update(&ctx)
        .await?;

    r.into_active_model().delete(&ctx).await?;

    ctx.commit().await?;

    ok!(())
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "mutation.ts")]
struct SetItemRequest {
    pub quantity: i32,
}

/// 需要对余额进行操作，根据数量变化，可能是返还余额，也可能是扣除余额
async fn set_item(
    State(AppState { db, config }): State<AppState>,
    Path(OperationRecord { stu_id, item_id }): Path<OperationRecord>,
    Json(SetItemRequest { quantity }): Json<SetItemRequest>,
) -> R {
    let quantity = Ord::max(quantity, 1);

    let conf = config.balance_config().await;

    let ctx = db.begin().await?;

    if let Some(mut r) = record::Entity::load()
        .filter(record::COLUMN.item_id.eq(item_id))
        .filter(record::COLUMN.student_id.eq(stu_id))
        .with(student::Entity)
        .one(&ctx)
        .await?
    {
        let stu = r.student.take().ok_or(BizErrKind::StuNotFound)?;

        let diff = quantity - r.quantity;
        let price = match conf.pay_for_original_price {
            true => r.original_price,
            false => r.discount_price,
        };

        let new_balance = stu.balance - Decimal::from(diff) * price;

        stu.into_active_model()
            .set_balance(new_balance)
            .update(&ctx)
            .await?;

        r.into_active_model()
            .set_quantity(quantity)
            .update(&ctx)
            .await?;
    } else {
        let item = item::Entity::find_by_id(item_id)
            .one(&ctx)
            .await?
            .ok_or(BizErrKind::ItemNotFound)?;

        let stu = student::Entity::find_by_id(stu_id)
            .one(&ctx)
            .await?
            .ok_or(BizErrKind::StuNotFound)?;

        let d_price = match stu.difficulty_level {
            Difficulty::Peaceful => item.price,
            Difficulty::Easy => item.p_easy,
            Difficulty::Normal => item.p_normal,
            Difficulty::Hard => item.p_hard,
        };

        let new_balance = stu.balance
            - Decimal::from(quantity)
                * match conf.pay_for_original_price {
                    true => item.price,
                    false => d_price,
                };

        record::ActiveModel::builder()
            .set_student_id(stu_id)
            .set_item_id(item_id)
            .set_student_no(stu.student_no.clone())
            .set_stu_d_level(stu.difficulty_level)
            .set_quantity(quantity)
            .set_original_price(item.price)
            .set_discount_price(d_price)
            .set_item_name(item.name)
            .set_item_spec(item.spec)
            .insert(&ctx)
            .await?;

        stu.into_active_model()
            .into_ex()
            .set_balance(new_balance)
            .update(&ctx)
            .await?;
    }

    ctx.commit().await?;

    ok!(())
}

async fn get_stu_record(State(ref db): State<DbConn>, Path(id): Path<i32>) -> R {
    let find = student::Entity::load()
        .filter_by_id(id)
        .with(record::Entity)
        .one(db)
        .await?
        .ok_or(BizErrKind::StuNotFound)?;

    let records = match find.records {
        HasMany::Loaded(records) => records,
        HasMany::Unloaded => Default::default(),
    };

    ok!(records)
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "summary.ts")]
struct StudentRecordWithSummary {
    records: Vec<record::ModelEx>,
    summary: StudentRecordSummary,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "summary.ts")]
struct StudentRecordSummary {
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    total_discount_price: Decimal,
    total_quantity: i32,
    category_count: usize,
}

async fn get_stu_record_with_summary(State(ref db): State<DbConn>, Path(id): Path<i32>) -> R {
    let find = student::Entity::load()
        .filter_by_id(id)
        .with(record::Entity)
        .one(db)
        .await?
        .ok_or(BizErrKind::StuNotFound)?;

    let records = match find.records {
        HasMany::Loaded(records) => records,
        HasMany::Unloaded => Default::default(),
    };

    let total_discount_price = records
        .iter()
        .map(|r| r.discount_price * Decimal::from(r.quantity))
        .sum::<Decimal>();

    let category_count = records
        .iter()
        .map(|r| r.item_id)
        .collect::<HashSet<_>>()
        .len();

    let total_quantity = records.iter().map(|r| r.quantity).sum();

    ok!(StudentRecordWithSummary {
        records,
        summary: StudentRecordSummary {
            total_discount_price,
            total_quantity,
            category_count
        }
    })
}

async fn get_stu(State(AppState { ref db, .. }): State<AppState>, Path(id): Path<i32>) -> R {
    let stu = student::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(BizErrKind::StuNotFound)?;

    ok!(stu)
}

pub async fn list(
    State(db): State<DbConn>,
    Query(OrderPagination {
        page,
        per_page,
        order_by_key,
        order_by_type,
        q,
    }): Query<OrderPagination>,
) -> R {
    let mut find = student::Entity::find();

    if let Some(q) = q {
        let q = q.replace("%", "");
        let q = q.split_whitespace();
        let mut cod = Condition::all();

        for qi in q {
            cod = cod.add(
                student::Column::Name
                    .contains(qi)
                    .or(student::Column::StudentNo.contains(qi))
                    .or(student::Column::SecondarySchool.contains(qi))
                    .or(student::Column::Class.contains(qi)),
            );
            let expr: SimpleExpr = Expr::case(
                Expr::col(student::Column::Name)
                    .like(format!("{}%", qi))
                    .or(Expr::col(student::Column::StudentNo).like(format!("{}%", qi))),
                Expr::val(0),
            )
            .finally(Expr::val(1))
            .into();
            find = find.order_by_asc(expr);
        }
        find = find.filter(cod);
    }

    if let (Some(key), Some(order)) = (order_by_key, order_by_type) {
        let col = student::Column::from_str(&key)?;
        find = find.order_by(col, order.into());
    }

    let p = find.paginate(&db, Ord::max(per_page, 1));

    ok!(PaginateData {
        page,
        per_page,
        length: p.num_items().await?,
        data: p.fetch_page(Ord::max(page, 1) - 1).await?,
    })
}
