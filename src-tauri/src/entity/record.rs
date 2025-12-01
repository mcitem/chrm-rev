use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{entity::student::Difficulty, utils::NextPrimaryKey};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./entity.ts")]
#[sea_orm(model_attrs(ts(rename = "Record")))]
#[sea_orm(model_ex_attrs(ts(rename = "RecordEx")))]
#[sea_orm(table_name = "record")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub student_id: i32,
    pub item_id: i32,
    pub student_no: String,
    pub stu_d_level: Difficulty,
    pub quantity: i32,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    /// 折后单价快照
    pub discount_price: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    /// 原价单价快照
    pub original_price: Decimal,
    pub item_name: String,
    pub item_spec: String,
    #[sea_orm(
        belongs_to,
        from = "item_id",
        to = "id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    #[serde(skip_serializing_if = "HasOne::is_none")]
    pub item: HasOne<super::item::Entity>,
    #[sea_orm(
        belongs_to,
        from = "student_id",
        to = "id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    #[serde(skip_serializing_if = "HasOne::is_none")]
    pub student: HasOne<super::student::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl NextPrimaryKey for Entity {
    fn primary_column() -> Self::Column {
        Column::Id
    }
}
