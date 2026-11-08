use axum::Router;

use crate::router::state::AppState;

pub mod item;
mod manage;
pub mod record;
pub mod student;

pub fn router(state: AppState) -> Router<()> {
    Router::new()
        .nest("/student", student::router())
        .nest("/record", record::router())
        .nest("/item", item::router())
        .nest("/manage", manage::router())
        .with_state(state)
}
