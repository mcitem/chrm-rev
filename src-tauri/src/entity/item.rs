use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utils::NextPrimaryKey;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./entity.ts")]
#[sea_orm(model_attrs(ts(rename = "Item")))]
#[sea_orm(model_ex_attrs(ts(rename = "ItemEx")))]
#[sea_orm(table_name = "item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub name: String,
    pub spec: String,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub price: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub p_easy: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub p_normal: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub p_hard: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub p_score: Decimal,
    #[sea_orm(has_many)]
    #[serde(skip_serializing_if = "HasMany::is_empty")]
    pub records: HasMany<super::record::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl NextPrimaryKey for Entity {
    fn primary_column() -> Self::Column {
        Column::Id
    }
}
