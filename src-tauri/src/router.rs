use axum::Router;
use sea_orm::DatabaseConnection;

use crate::{
    config::Config,
    router::state::{AppState, ComplexState},
};

pub(self) mod prelude;

mod response;
pub mod state;

pub mod utils;

pub mod biz;

mod sys;

pub fn app(
    config: Config,
    db: DatabaseConnection,
    app: tauri::AppHandle<impl tauri::Runtime>,
) -> Router<()> {
    let state = AppState {
        db: db.clone(),
        config,
    };
    let complex = ComplexState { db, app };

    Router::new()
        .nest("/sys", sys::router(complex))
        .nest("/biz", biz::router(state.clone()))
}
