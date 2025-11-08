use sea_orm::entity::prelude::*;
use serde::Serialize;
use ts_rs::TS;

use crate::utils::NextPrimaryKey;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, TS, Serialize)]
#[sea_orm(table_name = "record")]
#[ts(rename = "Record")]
#[ts(export, export_to = "./entity.ts")]
/// Record Model
/// 除了id和数量其余都是冗余字段
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub student_id: i32,
    pub item_id: i32,
    pub student_no: String,
    pub stu_d_level: String,
    pub quantity: i32,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub discount_price: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub original_price: Decimal,
    pub item_name: String,
    pub item_spec: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::item::Entity",
        from = "Column::ItemId",
        to = "super::item::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Item,
    #[sea_orm(
        belongs_to = "super::student::Entity",
        from = "Column::StudentId",
        to = "super::student::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Student,
}

impl Related<super::item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Item.def()
    }
}

impl Related<super::student::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Student.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl NextPrimaryKey for Entity {
    fn primary_column() -> Self::Column {
        Column::Id
    }
}
