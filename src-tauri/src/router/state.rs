use axum::extract::FromRef;
use sea_orm::DatabaseConnection;

#[derive(Clone, Default)]

pub(super) struct AppState {
    pub db: DatabaseConnection,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}
