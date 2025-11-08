use crate::entity::{item, student};
use rust_xlsxwriter::{Workbook, XlsxError, worksheet::Worksheet};
use sea_orm::{ConnectionTrait, EntityTrait};

pub async fn export_all<C: ConnectionTrait, P: AsRef<std::path::Path>>(
    conn: &C,
    path: P,
) -> Result<(), ExportError> {
    // let mut workbook = Workbook::new();
    // let mut stu_sheet = workbook.add_worksheet();

    // let mut item_sheet = Worksheet::new().set_name("item")?;

    // stu_sheet

    let mut stu_sheet = Worksheet::new();
    stu_sheet.set_name("stu")?;
    stu_sheet.write_row(
        0,
        0,
        ["学号", "姓名", "认定级别", "学院", "班级", "性别", "余额"],
    )?;
    let students = student::Entity::find().all(conn).await?;
    for (i, stu) in students.iter().enumerate() {
        let row = (i + 1) as u32;
        stu_sheet.write_string(row, 0, &stu.student_no)?;
        stu_sheet.write_string(row, 1, &stu.name)?;
        stu_sheet.write_string(row, 2, &stu.difficulty_level)?;
        if let Some(sec_sc) = &stu.secondary_school {
            stu_sheet.write_string(row, 3, sec_sc)?;
        }
        if let Some(class) = &stu.class {
            stu_sheet.write_string(row, 4, class)?;
        }
        if let Some(sex) = &stu.sex {
            stu_sheet.write_string(row, 5, sex)?;
        }
        if let Some(balance) = stu.balance {
            stu_sheet.write_string(row, 6, balance.to_string())?;
        }
    }

    let mut item_sheet = Worksheet::new();
    item_sheet.set_name("item")?;
    item_sheet.write_row(
        0,
        0,
        [
            "商品名称",
            "规格",
            "原价",
            "3折价",
            "5折价",
            "7折价",
            "积分",
        ],
    )?;
    let items = item::Entity::find().all(conn).await?;
    for (i, item) in items.iter().enumerate() {
        let row = (i + 1) as u32;
        item_sheet.write_string(row, 0, &item.name)?;
        item_sheet.write_string(row, 1, item.spec.to_string())?;
        item_sheet.write_string(row, 2, item.price.to_string())?;
        item_sheet.write_string(row, 3, item.p_hard.to_string())?;
        item_sheet.write_string(row, 4, item.p_normal.to_string())?;
        item_sheet.write_string(row, 5, item.p_easy.to_string())?;
        item_sheet.write_string(row, 6, item.p_score.to_string())?;
    }

    let mut workbook = Workbook::new();
    workbook.push_worksheet(stu_sheet);
    workbook.push_worksheet(item_sheet);

    workbook.save(path)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ExportError {
    #[error("{0}")]
    DbSeaOrm(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Xlsx(#[from] rust_xlsxwriter::XlsxError),
}
