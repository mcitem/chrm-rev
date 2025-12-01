use sea_orm::ExprTrait;

use crate::router::prelude::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/all", get(all))
        .route("/list", get(list))
        .route("/{id}", get(get_item).put(on_put).delete(on_delete))
}

type R = Result<AppResponse<serde_json::Value>, BizErrKind>;

async fn all(State(ref db): State<DbConn>) -> R {
    let r = item::Entity::find().all(db).await?;
    ok!(r)
}

async fn get_item(State(ref db): State<DbConn>, Path(id): Path<i32>) -> R {
    let r = item::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(BizErrKind::ItemNotFound)?;
    ok!(r)
}

async fn on_put(
    State(ref db): State<DbConn>,
    Path(_): Path<i32>,
    Json(item): Json<serde_json::Value>,
) -> R {
    let item = item::ActiveModel::from_json(item)?;
    let res: item::Model = item.update(db).await?;

    ok!(res)
}

async fn on_delete(State(ref db): State<DbConn>, Path(id): Path<i32>) -> R {
    let res = item::Entity::delete_by_id(id).exec(db).await?;
    ok!(res.rows_affected)
}

/// 商品搜索接口
async fn list(
    State(ref db): State<DbConn>,
    Query(OrderPagination {
        page,
        per_page,
        order_by_key,
        order_by_type,
        q,
    }): Query<OrderPagination>,
) -> R {
    let mut find = item::Entity::find();

    if let Some(q) = q {
        let q = q.trim_start_matches("0");
        let q = q.replace("%", "");
        let q = q.split_whitespace();

        let mut cod = Condition::all();

        for qi in q {
            let prefix_q = format!("{}%", qi);
            cod = cod.add(
                item::Column::Name
                    .contains(qi)
                    .or(item::Column::Id.contains(qi)),
            );
            let expr: SimpleExpr =
                Expr::case(Expr::col(item::Column::Id).like(&prefix_q), Expr::val(2))
                    .case(Expr::col(item::Column::Name).like(&prefix_q), Expr::val(1))
                    .finally(Expr::val(0))
                    .into();
            // 降序
            find = find.order_by_desc(expr);
        }
        find = find.filter(cod);
    }

    if let (Some(key), Some(order)) = (order_by_key, order_by_type) {
        let col = item::Column::from_str(&key)?;
        find = find.order_by(col, order.into());
    }

    let p = find.paginate(db, Ord::max(per_page, 1));

    ok!(PaginateData {
        page,
        per_page,
        length: p.num_items().await?,
        data: p.fetch_page(Ord::max(page, 1) - 1).await?,
    })
}
