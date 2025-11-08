use prelude::*;
use sea_orm::DatabaseConnection;
use state::AppState;

/// 用于 handle 的预导入
pub(self) mod prelude;

/// 系统接口
mod sys;

/// 迁移状态接口
mod migration;
/// 响应封装工具
mod response;
/// 状态管理
mod state;
/// 其他工具
mod utils;

/// item本身接受
mod item;
/// 记录相关接口
mod record;

/// 学生接口
mod student;

pub fn app(db: DatabaseConnection) -> Router<()> {
    let state = AppState { db };

    Router::new()
        .route("/", get(async || "hello"))
        .nest("/sys", sys::router())
        .nest("/migration", migration::router(state.clone()))
        .nest("/item", item::router(state.clone()))
        .nest("/student", student::router(state.clone()))
        .nest("/record", record::router(state.clone()))
        .with_state(state)
}

// async fn get_state(State(s): State<AppState>) {}
