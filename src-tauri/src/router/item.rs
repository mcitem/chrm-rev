use std::str::FromStr;

use axum::{Json, extract::Query};
use sea_orm::{Condition, QueryOrder, sea_query::SimpleExpr};

use crate::router::{response::AppErrExt, utils::PaginateData};

use super::{prelude::*, utils::OrderPagination};

pub fn router(_: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(on_post))
        .route("/list", get(list))
        .route("/{id}", put(on_put).delete(on_delete))
    // .route("/resetSoldCount", get(reset_sold_count))
}

#[deprecated]
async fn reset_sold_count(State(ref db): State<DbConn>) -> R {
    let res = item::Entity::update_many()
        // .col_expr(item::Column::SoldEasy, Expr::value(0))
        // .col_expr(item::Column::SoldNormal, Expr::value(0))
        // .col_expr(item::Column::SoldHard, Expr::value(0))
        .exec(db)
        .await?;
    ok!(res.rows_affected)
}

async fn on_post(State(ref db): State<DbConn>, Json(item): Json<serde_json::Value>) -> R {
    let item = item::ActiveModel::from_json(item)?;
    // println!("{:?}", item);
    let res = item.insert(db).await?;
    ok!(res)
}

async fn on_put(
    State(ref db): State<DbConn>,
    Path(_): Path<i32>,
    Json(item): Json<serde_json::Value>,
) -> R {
    let item = item::ActiveModel::from_json(item)?;
    // println!("{:?}", item);
    let res = item.update(db).await?;
    ok!(res)
}

async fn on_delete(State(ref db): State<DbConn>, Path(id): Path<i32>) -> R {
    let res = item::Entity::delete_by_id(id).exec(db).await?;
    ok!(res.rows_affected)
}

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
    // tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    let mut find = item::Entity::find();

    // if let Some(q) = q {
    //     find = find.filter(
    //         Condition::any()
    //             .add(item::Column::Name.contains(&q))
    //             .add(item::Column::Spec.contains(&q))
    //             .add(item::Column::Id.contains(&q)),
    //     );
    // }

    if let Some(q) = q {
        let q = q.split_whitespace();
        let mut cod = Condition::all();

        for qi in q {
            cod = cod.add(
                item::Column::Name
                    .contains(qi)
                    .or(item::Column::Spec.contains(qi))
                    .or(item::Column::Id.contains(qi)),
                // .or(item::Column::Id.contains(qi.trim_prefix("#"))),
            );
            let expr: SimpleExpr = Expr::case(
                Expr::col(item::Column::Id)
                    .like(format!("{}%", qi))
                    .or(Expr::col(item::Column::Name).like(format!("{}%", qi))),
                Expr::val(0),
            )
            .finally(Expr::val(1))
            .into();
            find = find.order_by_asc(expr);
        }
        find = find.filter(cod);
    }

    if let (Some(key), Some(order)) = (order_by_key, order_by_type) {
        let col = item::Column::from_str(&key).map_err(|e| e.to_string().e_400())?;
        find = find.order_by(col, order.into());
    }

    let p = find.paginate(db, per_page.max(1));

    ok!(PaginateData {
        page,
        per_page,
        length: p.num_items().await?,
        data: p.fetch_page(page.max(1) - 1).await?,
    })
}
