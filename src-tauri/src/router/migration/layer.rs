use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use futures::TryFutureExt;
use futures::future::BoxFuture;
use sea_orm_migration::MigratorTrait;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::{
    migration::Migrator,
    router::{AppState, response::AppErrExt},
};

#[derive(Clone)]
/// 迁移中间件，用于检查是否有未完成的迁移，如迁移已完成则继续处理请求
/// 限用于 .router_layer 仅限路由匹配layer
/// 以免在404浪费
/// todo: 简单验证
pub(super) struct MigrateUpLayer(pub AppState);

impl<S> Layer<S> for MigrateUpLayer {
    type Service = MigrateUpService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MigrateUpService {
            inner,
            state: self.0.clone(),
        }
    }
}

#[derive(Clone)]
pub(super) struct MigrateUpService<S> {
    inner: S,
    state: AppState,
}

impl<S> Service<Request> for MigrateUpService<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let f = self.inner.call(req);
        let db = self.state.db.clone();

        Box::pin(
            async move {
                let s = Migrator::get_pending_migrations(&db).await?;
                tracing::debug!("pending migrations: {}", s.len());
                if s.is_empty() {
                    return "不存在未完成的迁移".e_400().map();
                };
                Ok(f.await)
            }
            .unwrap_or_else(|e| Ok(e.into_response())),
        )
    }
}
