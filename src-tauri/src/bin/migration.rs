use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    cli::run_cli(chrm_rev_lib::migration::Migrator).await;
}
