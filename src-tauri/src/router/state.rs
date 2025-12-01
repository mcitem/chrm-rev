use axum::extract::FromRef;
use sea_orm::DatabaseConnection;
use tauri::{AppHandle, Runtime};

use crate::config::Config;

pub struct ComplexState<RT: Runtime> {
    pub db: DatabaseConnection,
    pub app: AppHandle<RT>,
}

impl<RT: Runtime> Clone for ComplexState<RT> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            app: self.app.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

impl<RT: Runtime> FromRef<ComplexState<RT>> for DatabaseConnection {
    fn from_ref(input: &ComplexState<RT>) -> Self {
        input.db.clone()
    }
}

impl<RT: Runtime> FromRef<ComplexState<RT>> for AppHandle<RT> {
    fn from_ref(input: &ComplexState<RT>) -> Self {
        input.app.clone()
    }
}
