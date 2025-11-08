use sea_orm_migration::prelude::*;

pub mod iden;

mod m0_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // 仅建表，初始化数据导入由服务完成
        vec![Box::new(m0_create_table::Migration)]
    }
}
