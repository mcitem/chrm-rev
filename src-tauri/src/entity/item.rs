use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utils::NextPrimaryKey;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[ts(rename = "Item")]
#[ts(export, export_to = "./entity.ts")]
#[sea_orm(table_name = "item")]
/// Item Model
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    /// 商品名称
    pub name: String,
    /// 商品规格
    pub spec: String,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    /// 原价
    pub price: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    /// 一般困难 价格
    pub p_easy: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    /// 困难 价格
    pub p_normal: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    /// 特别困难 价格
    pub p_hard: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    /// 积分兑换 价格
    pub p_score: Decimal,
    // // #[sea_orm(default_value = 0)]
    // pub sold_easy: i32,
    // // #[sea_orm(default_value = 0)]
    // pub sold_normal: i32,
    // // #[sea_orm(default_value = 0)]
    // pub sold_hard: i32,
    /// 标签
    pub tags: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
//     fn new() -> Self {
//         use sea_orm::ActiveValue::NotSet;
//         Self {
//             id: NotSet,
//             name: NotSet,
//             spec: NotSet,
//             price: NotSet,
//             p_easy: NotSet,
//             p_hard: NotSet,
//             p_normal: NotSet,
//             p_score: NotSet,
//             // sold_easy: Set(0),
//             // sold_normal: Set(0),
//             // sold_hard: Set(0),
//             tags: NotSet,
//         }
//     }
// }

impl NextPrimaryKey for Entity {
    fn primary_column() -> Self::Column {
        Column::Id
    }
}
