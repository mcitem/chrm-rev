use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("CREATE TABLE app_state (id INTEGER PRIMARY KEY CHECK(id=1), data_ready BOOLEAN NOT NULL)").await?;
        db.execute_unprepared("INSERT INTO app_state SELECT 1, EXISTS(SELECT 1 FROM student) OR EXISTS(SELECT 1 FROM item)").await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE app_state").await?;
        Ok(())
    }
}
