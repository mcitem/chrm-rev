use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Insert, Iterable, TransactionTrait,
};

const BIND_BUDGET: usize = 900;

pub(super) async fn insert_in_batches<A: ActiveModelTrait>(
    db: &DatabaseConnection,
    models: Vec<A>,
) -> Result<usize, DbErr> {
    let count = models.len();
    if count == 0 {
        return Ok(0);
    }
    let columns = <A::Entity as EntityTrait>::Column::iter().count().max(1);
    let batch_size = (BIND_BUDGET / columns).max(1);
    let mut models = models.into_iter();
    let txn = db.begin().await?;
    loop {
        let batch: Vec<_> = models.by_ref().take(batch_size).collect();
        if batch.is_empty() {
            break;
        }
        if let Err(error) = Insert::many(batch).exec_without_returning(&txn).await {
            txn.rollback().await?;
            return Err(error);
        }
    }
    txn.commit().await?;
    Ok(count)
}
