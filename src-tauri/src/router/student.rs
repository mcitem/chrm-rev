use axum::{Json, extract::Query};
use sea_orm::{ActiveValue, Condition, IntoActiveModel, QueryOrder, sea_query::SimpleExpr};
use std::str::FromStr;

use crate::{
    entity::student::{self, Difficulty},
    router::response::AppErrExt,
    utils::NextPrimaryKey,
};

use super::{
    prelude::*,
    utils::{OrderPagination, PaginateData},
};

pub fn router(_: AppState) -> Router {
    Router::new()
        .route("/", post(on_post))
        .route("/list", get(list))
        .route("/{id}", get(get_stu).delete(on_delete).put(on_put))
        .route("/{id}/touch", get(on_touch))
}

async fn get_stu(State(ref db): State<DbConn>, Path(id): Path<i32>) -> R {
    let stu = student::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or("学生不存在".e_404())?;
    ok!(stu)
}

/// 初始化学生余额（在添加tab时触发）
async fn on_touch(State(ref db): State<DbConn>, Path(id): Path<i32>) -> R {
    let stu = student::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or("学生不存在".e_404())?;
    let stu = if Config::balance_config().await.active && stu.balance.is_none() {
        let mut stu = stu.into_active_model();

        if let ActiveValue::Unchanged(None) = stu.balance {
            let level = Difficulty::from_str(stu.difficulty_level.as_ref());
            stu.balance = Set(Some(level.as_balance().await));
        }

        let res = stu.update(db).await?;
        res
    } else {
        stu
    };
    ok!(stu)
}

async fn on_post(State(ref db): State<DbConn>, Json(student): Json<student::Model>) -> R {
    let stu = student::ActiveModel {
        id: Set(student::Entity::next_pk(db).await?),
        name: Set(student.name),
        student_no: Set(student.student_no),
        difficulty_level: Set(Difficulty::from_str(&student.difficulty_level).to_string()),
        secondary_school: Set(student.secondary_school),
        class: Set(student.class),
        sex: Set(student.sex),
        balance: Set(student.balance),
    };
    student::Entity::insert(stu).exec(db).await?;
    ok!(())
}

async fn on_put(
    State(ref db): State<DbConn>,
    Path(id): Path<i32>,
    Json(student): Json<student::Model>,
) -> R {
    let stu = student::ActiveModel {
        id: Set(id),
        name: Set(student.name),
        student_no: Set(student.student_no),
        difficulty_level: Set(Difficulty::from_str(&student.difficulty_level).to_string()),
        secondary_school: Set(student.secondary_school),
        class: Set(student.class),
        sex: Set(student.sex),
        balance: Set(student.balance),
    };

    let res = student::Entity::update(stu).exec(db).await?;

    ok!(res)
}
async fn on_delete(State(ref db): State<DbConn>, Path(id): Path<i32>) -> R {
    let res = student::Entity::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err("学生不存在".e_404());
    }
    ok!(())
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
    let mut find = student::Entity::find();

    if let Some(q) = q {
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

    let p = find.paginate(db, per_page.max(1));

    ok!(PaginateData {
        page,
        per_page,
        length: p.num_items().await?,
        data: p.fetch_page(page.max(1) - 1).await?,
    })
}
