use std::path::PathBuf;

use sea_orm::{
    ColumnTrait, ConnectOptions, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, QuerySelect,
};

pub struct SqlitePath(pub PathBuf);

impl Into<ConnectOptions> for SqlitePath {
    fn into(self) -> ConnectOptions {
        let mut opt = ConnectOptions::new(format!("sqlite://{}?mode=rwc", self.0.display()));
        opt.sqlx_logging(false);
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

        find.and_then(|r| r.max_id)
            .unwrap_or(0)
            .max(0)
            .checked_add(1)
            .ok_or_else(|| DbErr::Custom("主键已超出 i32 范围".into()))
    }
}

pub async fn data_ready(db: &impl ConnectionTrait) -> Result<bool, DbErr> {
    let row = db
        .query_one_raw(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Sqlite,
            "SELECT data_ready FROM app_state WHERE id = 1",
        ))
        .await?;
    Ok(row
        .map(|r| r.try_get::<bool>("", "data_ready"))
        .transpose()?
        .unwrap_or(false))
}

pub async fn set_data_ready(db: &impl ConnectionTrait, ready: bool) -> Result<(), DbErr> {
    db.execute_raw(sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Sqlite,
        "UPDATE app_state SET data_ready = ? WHERE id = 1",
        [ready.into()],
    ))
    .await?;
    Ok(())
}
