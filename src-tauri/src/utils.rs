use std::path::PathBuf;

use sea_orm::{
    ColumnTrait, ConnectOptions, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, QuerySelect,
};

pub struct SqlitePath(pub PathBuf);

impl Into<ConnectOptions> for SqlitePath {
    fn into(self) -> ConnectOptions {
        let opt = ConnectOptions::new(format!("sqlite://{}?mode=rwc", self.0.display()));
        // opt.sqlx_logging(false);
        // .connect_timeout(Duration::from_secs(5));
        opt
    }
}

pub trait NextPrimaryKey: EntityTrait {
    fn primary_column() -> Self::Column;

    async fn next_pk<C>(conn: &C) -> Result<i32, DbErr>
    where
        C: ConnectionTrait,
    {
        #[derive(Debug, FromQueryResult)]
        struct SelectResult {
            pub max_id: Option<i32>,
        }

        let find: Option<SelectResult> = Self::find()
            .select_only()
            .column_as(Self::primary_column().max(), "max_id")
            .into_model()
            .one(conn)
            .await?;

        Ok(find.and_then(|r| r.max_id).unwrap_or(1))
    }
}
