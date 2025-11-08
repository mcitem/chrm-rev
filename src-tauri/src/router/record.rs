use axum::Json;
use sea_orm::{IntoActiveModel, TransactionTrait};

use crate::{
    config::CONFIG,
    entity::{record, student::Difficulty},
    router::response::AppErr,
    utils::{NextPrimaryKey, advance_export, export_gbk},
};

use super::prelude::*;

pub fn router(_: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_record).delete(delect_all_record))
        .route("/export", post(export))
        .route("/{stu_id}/item", get(get_record_item))
        .route(
            "/{stu_id}/item/{item_id}",
            delete(delete_item).post(set_item),
        )
}

/// 直接删除所有记录，不返回余额，仅在导出后使用
async fn delect_all_record(State(ref db): State<DbConn>) -> R {
    record::Entity::delete_many().exec(db).await?;
    ok!(())
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "mutation.ts")]
struct ExportRequest {
    pub sign: String,
}

/// 根据配置导出记录到指定路径
/// todo:优化报不报错
async fn export(State(s): State<AppState>, Json(req): Json<ExportRequest>) -> R {
    let export_conf = &CONFIG.read().await.export_config;

    if let Some(p) = &export_conf.path {
        export_gbk(p, &s.db, &req.sign).await?;
    }

    if let Some(p) = &export_conf.advance_path {
        advance_export::advance_export(p, &s.db, &req.sign)
            .await
            .map_err(|e| AppErr::e_400(format!("xlsx导出错误{}", e.to_string())))?;
    };

    ok!(())
}

/// 获取所有记录，在首页展示使用
async fn get_all_record(State(ref db): State<DbConn>) -> R {
    let r = record::Entity::find().all(db).await?;
    ok!(r)
}

/// 对record进行操作的参数
#[derive(Deserialize, TS)]
#[ts(export, export_to = "mutation.ts")]
struct OperationRecord {
    pub item_id: i32,
    pub stu_id: i32,
}

/// 删除学生记录
/// 需要返还余额
async fn delete_item(
    State(ref db): State<DbConn>,
    Path(OperationRecord { stu_id, item_id }): Path<OperationRecord>,
) -> R {
    let conf = Config::balance_config().await;

    let ctx = db.begin().await?;
    let db = &ctx;

    let rec_i = record::Entity::find()
        .filter(record::Column::StudentId.eq(stu_id))
        .filter(record::Column::ItemId.eq(item_id))
        .one(db)
        .await?;

    if let Some(ri) = rec_i {
        if conf.active {
            // 余额是否按原价计算
            let return_credit = Decimal::from(ri.quantity)
                * match conf.pay_for_original_price {
                    true => ri.original_price,
                    false => ri.discount_price,
                };

            // 返还余额
            let stu = crate::entity::student::Entity::find_by_id(stu_id)
                .one(db)
                .await?
                .ok_or_else(|| AppErr::e_400("该学生不存在"))?
                .init_balance(db)
                .await?;

            let ori_credit = stu.balance.unwrap_or_default();

            let new_credit = ori_credit + return_credit;

            let mut act = stu.into_active_model();

            act.balance = Set(Some(new_credit));

            act.update(db).await?;
        };

        ri.delete(db).await?;
    }

    ctx.commit().await?;
    ok!(())
}

async fn get_record_item(Path(stu_id): Path<i32>, State(s): State<AppState>) -> R {
    let r = record::Entity::find()
        .filter(record::Column::StudentId.eq(stu_id))
        .all(&s.db)
        .await?;

    ok!(r)
}

#[derive(Deserialize)]
struct SetItemRequest {
    pub quantity: i32,
}

/// 需要对余额进行操作，根据数量变化，可能是返还余额，也可能是扣除余额
async fn set_item(
    State(ref db): State<DbConn>,
    Path(OperationRecord { stu_id, item_id }): Path<OperationRecord>,
    Json(SetItemRequest { quantity }): Json<SetItemRequest>,
) -> R {
    let quantity = quantity.max(1);

    let conf = Config::balance_config().await;

    let ctx = db.begin().await?;
    let db = &ctx;

    // 查询是否已有记录
    let rec = record::Entity::find()
        .filter(record::Column::StudentId.eq(stu_id))
        .filter(record::Column::ItemId.eq(item_id))
        .one(db)
        .await?;

    let result = match rec {
        Some(v) => {
            // 当已有记录时，更新记录

            if conf.active {
                // 数量变化，余额也要变化
                // v原始值,quantity新值
                let diff = Decimal::from(quantity) - Decimal::from(v.quantity);

                // 由于外键约束，学生一定存在
                // 但余额可能为空
                let stu = crate::entity::student::Entity::find_by_id(stu_id)
                    .one(db)
                    .await?
                    .ok_or_else(|| AppErr::e_400("该学生不存在"))?
                    .init_balance(db)
                    .await?;
                // 安全的unwrap
                let ori_balance = stu.balance.unwrap_or_default();

                // 余额是否按原价计算
                let new_balance = ori_balance
                    - diff
                        * match conf.pay_for_original_price {
                            true => v.original_price,
                            false => v.discount_price,
                        };

                if new_balance < -(conf.tolerance.abs()) && !conf.ignore_tolerance {
                    return Err(AppErr::e_400("余额不足，无法增加购买数量"));
                }

                // 更新余额
                let mut act_stu = stu.into_active_model();
                act_stu.balance = Set(Some(new_balance));
                act_stu.update(db).await?;
            }
            // 更新数量记录
            let mut act = v.into_active_model();
            act.quantity = Set(quantity);

            act.update(db).await
        }
        None => {
            // 当没有记录时，新增记录

            // 由于外键约束，学生和商品一定存在
            let stu = crate::entity::student::Entity::find_by_id(stu_id)
                .one(db)
                .await?
                .ok_or_else(|| AppErr::e_400("该学生不存在"))?;

            let item = crate::entity::item::Entity::find_by_id(item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppErr::e_400("商品不存在"))?;

            // 扣除余额
            let stu = if conf.active {
                let stu = stu.init_balance(db).await?;
                let ori_credit = stu.balance.unwrap_or_default();
                let new_credit = ori_credit - Decimal::from(quantity) * item.price;

                if new_credit < -(conf.tolerance.abs()) && !conf.ignore_tolerance {
                    return Err(AppErr::e_400("余额不足，无法购买"));
                }

                let mut act_stu = stu.into_active_model();
                act_stu.balance = Set(Some(new_credit));
                act_stu.update(db).await?
            } else {
                stu
            };

            let d_level = Difficulty::from_str(&stu.difficulty_level);
            let discount_price = match d_level {
                Difficulty::Peaceful => item.price,
                Difficulty::Easy => item.p_easy,
                Difficulty::Normal => item.p_normal,
                Difficulty::Hard => item.p_hard,
            };

            record::ActiveModel {
                id: Set(record::Entity::next_pk(db).await?),
                student_id: Set(stu_id),
                item_id: Set(item_id),
                quantity: Set(quantity),
                student_no: Set(stu.student_no),
                stu_d_level: Set(stu.difficulty_level),
                original_price: Set(item.price),
                discount_price: Set(discount_price),
                item_name: Set(item.name),
                item_spec: Set(item.spec),
            }
            .insert(db)
            .await
        }
    }?;
    ctx.commit().await?;
    ok!(result)
}
