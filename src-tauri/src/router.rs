use axum::Router;
use sea_orm::DatabaseConnection;
use tower::ServiceBuilder;
use tower_http::trace::{MakeSpan, TraceLayer};

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
        .route_layer(
            ServiceBuilder::new().layer(
                TraceLayer::new_for_http()
                    .make_span_with(TraceMakeSpan)
                    .on_request(())
                    .on_response(())
                    .on_body_chunk(())
                    .on_eos(()),
            ),
        )
}

#[derive(Debug, Clone)]
struct TraceMakeSpan;

impl<B> MakeSpan<B> for TraceMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        let uri = request.uri().path();

        if matches!(uri, "/sys/time") {
            return tracing::Span::none();
        }

        tracing::span!(
            tracing::Level::INFO,
            "request",
            method = %request.method(),
            uri = %uri,
        )
    }
}
