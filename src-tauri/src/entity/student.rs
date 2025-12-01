use sea_orm::entity::prelude::*;

use crate::{config::DefaultBalance, utils::NextPrimaryKey};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, str::FromStr};
use ts_rs::TS;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[ts(export, export_to = "./entity.ts")]
#[sea_orm(model_attrs(ts(rename = "Student")))]
#[sea_orm(model_ex_attrs(ts(rename = "StudentEx")))]
#[sea_orm(table_name = "student")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub name: String,
    pub student_no: String,
    pub difficulty_level: Difficulty,
    pub secondary_school: Option<String>,
    pub class: Option<String>,
    pub sex: Option<String>,
    pub major: Option<String>,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub balance: Decimal,
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

#[derive(
    Debug, Clone, TS, PartialEq, Eq, DeriveValueType, Serialize, Deserialize, Default, Copy,
)]
#[ts(export, export_to = "./types.ts")]
#[sea_orm(value_type = "String")]
/// 认定困难级别 eg: 一般困难, 困难, 特别困难
pub enum Difficulty {
    #[serde(rename = "不困难")]
    #[default]
    Peaceful,
    #[serde(rename = "一般困难")]
    Easy,
    #[serde(rename = "困难")]
    Normal,
    #[serde(rename = "特别困难")]
    Hard,
}

impl Difficulty {
    pub fn as_balance(&self, d: &DefaultBalance) -> Decimal {
        match self {
            Difficulty::Peaceful => d.peaceful_balance,
            Difficulty::Easy => d.easy_balance,
            Difficulty::Normal => d.normal_balance,
            Difficulty::Hard => d.hard_balance,
        }
    }
}

impl FromStr for Difficulty {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "一般困难" => Ok(Difficulty::Easy),
            "困难" => Ok(Difficulty::Normal),
            "特别困难" => Ok(Difficulty::Hard),
            _ => Ok(Difficulty::Peaceful),
        }
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Difficulty::Peaceful => "不困难",
            Difficulty::Easy => "一般困难",
            Difficulty::Normal => "困难",
            Difficulty::Hard => "特别困难",
        };
        write!(f, "{}", s)
    }
}

impl<'a> From<&'a str> for Difficulty {
    fn from(value: &'a str) -> Self {
        match value {
            "一般困难" => Difficulty::Easy,
            "困难" => Difficulty::Normal,
            "特别困难" => Difficulty::Hard,
            _ => Difficulty::Peaceful,
        }
    }
}

impl From<String> for Difficulty {
    fn from(value: String) -> Self {
        Difficulty::from(value.as_str())
    }
}
