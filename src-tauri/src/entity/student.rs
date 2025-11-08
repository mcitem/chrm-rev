use crate::{config::CONFIG, utils::NextPrimaryKey};
use sea_orm::{ActiveValue::Set, IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize, Serializer, de::Visitor};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "student")]
#[ts(rename = "Student")]
#[ts(export, export_to = "./entity.ts")]
/// Student Model

pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    /// 主键 手动自增
    pub id: i32,
    /// 姓名
    pub name: String,
    /// 学号
    pub student_no: String,
    /// 认定级别
    #[ts(as = "Difficulty")]
    pub difficulty_level: String,
    /// 学院
    pub secondary_school: Option<String>,
    /// 行政班级
    pub class: Option<String>,
    /// 性别
    pub sex: Option<String>,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    #[serde(with = "rust_decimal::serde::str_option")]
    #[ts(type = "string | null")]
    /// 已用额度
    pub balance: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::record::Entity")]
    Record,
}

impl Related<super::record::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Record.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl NextPrimaryKey for Entity {
    fn primary_column() -> Self::Column {
        Column::Id
    }
}

#[derive(Debug, Clone, TS, PartialEq, Eq)]
#[ts(export, export_to = "./types.ts")]
/// 认定困难级别 eg: 一般困难, 困难, 特别困难
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    // 兼容旧版，使用中文存储
    pub fn from_str(s: impl AsRef<str>) -> Self {
        match s.as_ref() {
            "Peaceful" | "不困难" => Self::Peaceful,
            "Easy" | "一般困难" => Self::Easy,
            "Normal" | "困难" => Self::Normal,
            "Hard" | "特别困难" => Self::Hard,
            _ => Self::Peaceful,
        }
    }
    pub const fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Peaceful => "不困难",
            Difficulty::Easy => "一般困难",
            Difficulty::Normal => "困难",
            Difficulty::Hard => "特别困难",
        }
    }

    pub async fn as_balance(&self) -> Decimal {
        let default = &CONFIG.read().await.balance_config.default_balance;
        match self {
            Difficulty::Peaceful => default.peaceful_balance,
            Difficulty::Easy => default.easy_balance,
            Difficulty::Normal => default.normal_balance,
            Difficulty::Hard => default.hard_balance,
        }
    }
}

impl ToString for Difficulty {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Serialize for Difficulty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Difficulty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DifficultyVisitor;

        impl<'de> Visitor<'de> for DifficultyVisitor {
            type Value = Difficulty;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("无效的困难级别类型")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Difficulty::from_str(value))
            }
        }

        deserializer.deserialize_str(DifficultyVisitor)
    }
}

impl Model {
    pub async fn init_balance(self, db: &impl ConnectionTrait) -> Result<Self, DbErr> {
        let stu = if None == self.balance {
            // 初始化余额逻辑
            let stu_level = Difficulty::from_str(&self.difficulty_level);
            let mut act = self.into_active_model();
            act.balance = Set(Some(stu_level.as_balance().await));
            act.update(db).await?
        } else {
            self
        };

        Ok(stu)
    }
}
