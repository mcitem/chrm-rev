use crate::config::DefaultBalance;
use crate::entity::student::Difficulty;
use crate::entity::{item, student};
use crate::router::response::error::system::DataloaderErrKind;
use crate::router::sys::dataloader::{LoadItemContext, LoadStuContext};
use calamine::{Range, Reader, Sheets};
use rust_decimal::Decimal;
use sea_orm::ActiveValue;
use sea_orm::Set;
use std::fs::File;
use std::str::FromStr;
use std::{io::BufReader, path::PathBuf};

pub fn check_path(path: String) -> Result<PathBuf, DataloaderErrKind> {
    let path = PathBuf::from(path);
    if !(path.is_absolute()
        && path.exists()
        && path.is_file()
        && path.extension().is_some_and(|s| s == "xlsx" || s == "xls"))
    {
        return Err(DataloaderErrKind::FileNotExecl(path.clone()))?;
    }
    Ok(path)
}

pub fn use_sheets(path: PathBuf) -> Result<Sheets<BufReader<File>>, DataloaderErrKind> {
    let sheets = calamine::open_workbook_auto(path).map_err(DataloaderErrKind::Calamine)?;
    Ok(sheets)
}

pub fn use_range_data(
    path: String,
    sheet: String,
) -> Result<Range<calamine::Data>, DataloaderErrKind> {
    let path = check_path(path)?;
    let mut sheets = use_sheets(path)?;
    let sheet_names = sheets.sheet_names();

    if !sheet_names.contains(&sheet) {
        return Err(DataloaderErrKind::SheetNotFound(sheet.clone()))?;
    }

    let sheet = sheets
        .worksheet_range(&sheet)
        .map_err(DataloaderErrKind::Calamine)?;

    Ok(sheet)
}

pub fn string(
    row: &[calamine::Data],
    index: usize,
    y: usize,
    data_include_header: usize,
    identifier: &'static str,
) -> Result<ActiveValue<String>, DataloaderErrKind> {
    Ok(row
        .get(index)
        .map(ToString::to_string)
        .map(ActiveValue::Set)
        .ok_or(DataloaderErrKind::FieldNotFound(
            identifier,
            index + 1,
            y + data_include_header + 1,
        ))?)
}

pub fn string_null(
    row: &[calamine::Data],
    index: Option<usize>,
    y: usize,
    data_include_header: usize,
    identifier: &'static str,
) -> Result<ActiveValue<Option<String>>, DataloaderErrKind> {
    Ok(match index {
        Some(i) => row
            .get(i)
            .map(ToString::to_string)
            .map(Some)
            .map(ActiveValue::Set)
            .ok_or(DataloaderErrKind::FieldNotFound(
                identifier,
                i + 1,
                y + data_include_header + 1,
            ))?,
        None => Set(None),
    })
}

pub fn decimal(
    row: &[calamine::Data],
    index: usize,
    y: usize,
    data_include_header: usize,
    identifier: &'static str,
) -> Result<ActiveValue<Decimal>, DataloaderErrKind> {
    Ok(row
        .get(index)
        .map(ToString::to_string)
        .map(|ref s| Decimal::from_str(s))
        .transpose()
        .map_err(|e| {
            DataloaderErrKind::Decimal(identifier, index + 1, y + data_include_header + 1, e)
        })?
        .map(ActiveValue::Set)
        .ok_or(DataloaderErrKind::FieldNotFound(
            identifier,
            index + 1,
            y + data_include_header + 1,
        ))?)
}

pub fn decimal_with_default(
    row: &[calamine::Data],
    index: Option<usize>,
    default: Decimal,
) -> ActiveValue<Decimal> {
    Set(index
        .and_then(|i| {
            row.get(i)
                .map(ToString::to_string)
                .and_then(|ref s| Decimal::from_str(s).ok())
        })
        .unwrap_or(default))
}

fn difficulty(
    row: &[calamine::Data],
    index: usize,
    y: usize,
    data_include_header: usize,
    identifier: &'static str,
) -> Result<Difficulty, DataloaderErrKind> {
    Ok(Difficulty::from(
        row.get(index)
            .map(ToString::to_string)
            .ok_or(DataloaderErrKind::FieldNotFound(
                identifier,
                index + 1,
                y + data_include_header + 1,
            ))?,
    ))
}

pub fn int(
    row: &[calamine::Data],
    index: Option<usize>,
    default: i32,
) -> Result<ActiveValue<i32>, DataloaderErrKind> {
    Ok(Set(index
        .and_then(|i| row.get(i))
        .map(ToString::to_string)
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(default)))
}

pub fn s_build_active_model(
    y: usize,
    row: &[calamine::Data],
    data_include_header: usize,
    ctx: &LoadStuContext,
    default_balance: DefaultBalance,
) -> Result<student::ActiveModel, DataloaderErrKind> {
    let y32 = i32::try_from(y + 1).map_err(DataloaderErrKind::LoadTryFromInt)?;

    let difficulty_level = difficulty(row, ctx.s_d_level, y, data_include_header, "学生困难等级")?;

    Ok(student::ActiveModel {
        id: int(row, ctx.id, y32)?,
        name: string(row, ctx.s_name, y, data_include_header, "学生姓名")?,
        student_no: string(row, ctx.s_no, y, data_include_header, "学生学号")?,
        difficulty_level: Set(difficulty_level),
        secondary_school: string_null(
            row,
            ctx.s_second_school,
            y,
            data_include_header,
            "学生第二就读学校",
        )?,
        sex: string_null(row, ctx.s_sex, y, data_include_header, "学生性别")?,
        class: string_null(row, ctx.s_class, y, data_include_header, "学生班级")?,
        balance: decimal_with_default(
            row,
            ctx.s_credit,
            difficulty_level.as_balance(&default_balance),
        ),
        major: string_null(row, ctx.s_major, y, data_include_header, "学生专业")?,
    })
}

pub fn i_build_active_model(
    y: usize,
    row: &[calamine::Data],
    data_include_header: usize,
    ctx: &LoadItemContext,
) -> Result<item::ActiveModel, DataloaderErrKind> {
    let y32 = i32::try_from(y + 1).map_err(DataloaderErrKind::LoadTryFromInt)?;

    Ok(item::ActiveModel {
        id: int(row, ctx.id, y32)?,
        name: string(row, ctx.i_name, y, data_include_header, "商品名称")?,
        spec: string(row, ctx.i_spec, y, data_include_header, "商品规格")?,
        price: decimal(row, ctx.i_p, y, data_include_header, "商品原价")?,
        p_hard: decimal(
            row,
            ctx.i_p_hard,
            y,
            data_include_header,
            "商品'特别困难'(5折价)价格",
        )?,
        p_normal: decimal(
            row,
            ctx.i_p_normal,
            y,
            data_include_header,
            "商品'困难'(5折价)价格",
        )?,
        p_easy: decimal(
            row,
            ctx.i_p_easy,
            y,
            data_include_header,
            "商品'一般困难'(7折价)价格",
        )?,
        p_score: decimal(row, ctx.i_p_score, y, data_include_header, "商品积分价格")?,
    })
}
