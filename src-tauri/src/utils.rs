use std::path::PathBuf;

use chrono::Timelike;
use rust_decimal::Decimal;
use sea_orm::{
    ConnectOptions, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryOrder,
};
use sea_orm_migration::async_trait;
use serde::Serializer;

pub struct SqlitePath(pub PathBuf);

impl Into<ConnectOptions> for SqlitePath {
    fn into(self) -> ConnectOptions {
        ConnectOptions::new(format!("sqlite://{}?mode=rwc", self.0.display()))
    }
}

#[async_trait::async_trait]
pub trait NextPrimaryKey: EntityTrait {
    fn primary_column() -> Self::Column;

    fn next_pk_err() -> DbErr {
        DbErr::Type(format!(
            "NextPrimaryKey: Table PK Column {:?} is not type i32",
            Self::primary_column()
        ))
    }

    async fn next_pk<C>(conn: &C) -> Result<i32, DbErr>
    where
        C: ConnectionTrait,
    {
        let select: sea_orm::Select<Self> = Self::find().order_by_desc(Self::primary_column());

        let res = select
            .one(conn)
            .await?
            .map(|v| match v.get(Self::primary_column()) {
                sea_orm::Value::Int(i) => match i {
                    Some(i) => Ok(i + 1),
                    None => Ok(1),
                },
                _ => Err(Self::next_pk_err()),
            })
            .transpose()?
            .unwrap_or(1);

        Ok(res)
    }
}

pub fn serialize_time<S: Serializer>(
    time: &chrono::NaiveTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if time.second() == 0 && time.nanosecond() == 0 {
        let s = format!("{:02}:{:02}", time.hour(), time.minute());
        serializer.serialize_str(&s)
    } else {
        serializer.serialize_str(&time.to_string())
    }
}

use encoding_rs::GBK;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::entity::record;

use thiserror::Error;
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("文件写入错误：{0}")]
    Io(#[from] tokio::io::Error),
    #[error("数据库错误{0}")]
    Db(#[from] DbErr),
}

// 1,2023112944,特别困难,24,3.6,12
// 2,2023017642,特别困难,2,3.3,11
// 3,2023017642,特别困难,22,2.4,8
// 4,2023112944,特别困难,7,4.2,14
// ,,,,13.5,
// 2025/9/21 17:00-18:30 张三,,,,,
// $$$$,,,,,

pub async fn export_gbk(
    path: impl AsRef<std::path::Path>,
    db: &DatabaseConnection,
    sign: &str,
) -> Result<(), ExportError> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .await?;

    let rec = record::Entity::find().all(db).await?;

    let mut total_discount_price = Decimal::new(0, 2);

    let mut count = 0;
    let mut out = String::new();
    rec.iter().for_each(|r| {
        for _ in 0..r.quantity {
            count += 1;
            out += &format!(
                "{},{},{},{},{},{}\r\n",
                count, r.student_no, r.stu_d_level, r.item_id, r.discount_price, r.original_price
            );
            total_discount_price += r.discount_price;
        }
    });

    let (bytes, _, _) = GBK.encode(&out);
    let (gbk_sign_bytes, _, _) = GBK.encode(sign);
    file.write_all(&bytes).await?;
    file.write_all(format!(",,,,{},\r\n", total_discount_price).as_bytes())
        .await?;
    // 签名行

    file.write_all(&gbk_sign_bytes).await?;
    file.write_all(b",,,,,\r\n").await?;

    file.write_all(b"$$$$,,,,,\r\n").await?;

    Ok(())
}

pub mod advance_export {
    use rust_decimal::Decimal;
    use sea_orm::{DatabaseConnection, EntityTrait};
    use umya_spreadsheet::Color;
    use umya_spreadsheet::reader::xlsx::lazy_read;
    use umya_spreadsheet::writer::xlsx::write;

    use crate::entity::record;

    // todo fix unwarp
    pub async fn advance_export(
        path: impl AsRef<std::path::Path>,
        db: &DatabaseConnection,
        sign: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let mut book = match path.exists() {
            true => lazy_read(path)?,
            false => umya_spreadsheet::new_file(),
        };
        let mut sheet = match book.get_sheet_mut(&0) {
            Some(s) => s,
            None => {
                book.new_sheet("Sheet1").unwrap();
                book.get_sheet_mut(&0).unwrap()
            }
        };

        let h_row = sheet.get_highest_row();

        // 写表头

        if h_row == 0 {
            sheet.get_cell_mut((1, 1)).set_value("序号");
            sheet.get_cell_mut((2, 1)).set_value("学号");
            sheet.get_cell_mut((3, 1)).set_value("商品编号");
            sheet.get_cell_mut((4, 1)).set_value("商品名称");
            sheet.get_cell_mut((5, 1)).set_value("规格");
            sheet.get_cell_mut((6, 1)).set_value("原价（进货单价）");
            sheet.get_cell_mut((7, 1)).set_value("折后价（出售单价）");
            sheet.get_cell_mut((8, 1)).set_value("数量（出售数量）");
        }

        // 写数据
        let rec = record::Entity::find().all(db).await?;

        let mut total_discount_price = Decimal::new(0, 2);

        let mut count = 0;
        for r in rec.iter() {
            count += 1;
            // 写一行
            let row = h_row + count;
            sheet.get_cell_mut((1, row)).set_value(count.to_string());

            sheet
                .get_cell_mut((2, row))
                .set_value_string(r.student_no.to_string());

            sheet
                .get_cell_mut((3, row))
                .set_value(r.item_id.to_string());

            sheet.get_cell_mut((4, row)).set_value(&r.item_name);

            sheet.get_cell_mut((5, row)).set_value(&r.item_spec);

            sheet
                .get_cell_mut((6, row))
                .set_value(r.original_price.to_string());

            sheet
                .get_cell_mut((7, row))
                .set_value(r.discount_price.to_string());

            sheet
                .get_cell_mut((8, row))
                .set_value(r.quantity.to_string());

            total_discount_price += r.discount_price * Decimal::new(r.quantity.into(), 0);
        }

        // 写分割行+样式

        let mut style = umya_spreadsheet::Style::default();

        style.set_background_color(Color::COLOR_YELLOW);

        // style.set

        let split_row = h_row + count + 1;
        sheet
            .get_cell_mut((1, split_row))
            .set_style(style.to_owned())
            .set_value_string("$$$$");

        sheet
            .get_cell_mut((2, split_row))
            .set_style(style.to_owned())
            .set_value_string(sign);
        sheet
            .get_cell_mut((3, split_row))
            .set_style(style.to_owned());
        sheet
            .get_cell_mut((4, split_row))
            .set_style(style.to_owned());
        sheet
            .get_cell_mut((5, split_row))
            .set_style(style.to_owned());
        sheet
            .get_cell_mut((6, split_row))
            .set_style(style.to_owned());

        sheet
            .get_cell_mut((7, split_row))
            .set_style(style.to_owned())
            .set_value_string(total_discount_price.to_string());

        sheet
            .get_cell_mut((8, split_row))
            .set_style(style.to_owned());

        // 写自动列宽
        sheet.get_column_dimensions_mut().iter_mut().for_each(|v| {
            v.set_auto_width(true);
        });

        write(&book, path)?;

        Ok(())
    }
}

// struct ExportManager<'a> {
//     db: &'a DatabaseConnection,
//     path: &'a std::path::Path,
//     sign: &'a str,
// }

// impl<'a> ExportManager<'a> {
//     pub fn new(db: &'a DatabaseConnection, path: &'a std::path::Path, sign: &'a str) -> Self {
//         Self { db, path, sign }
//     }
// }
