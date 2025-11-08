use crate::{
    entity::{item, student},
    router::response::{AppErr, AppErrExt},
    utils::NextPrimaryKey,
};

use calamine::Data;
use rust_decimal::Decimal;
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait, Set};
use serde::Serialize;
use std::str::FromStr;
use ts_rs::TS;

#[derive(Clone, Copy, Serialize, TS)]
#[ts(export, export_to = "./import.ts")]
pub struct SContext {
    s_name: usize,
    s_no: usize,
    s_d_level: usize,
    s_second_school: Option<usize>,
    s_class: Option<usize>,
    s_sex: Option<usize>,
    s_credit: Option<usize>,
}
#[derive(Clone, Copy, Serialize, TS)]
#[ts(export, export_to = "./import.ts")]
pub struct IContext {
    i_name: usize,
    i_spec: usize,
    i_p_easy: usize,
    i_p_normal: usize,
    i_p_hard: usize,
    i_p: usize,
    i_p_score: usize,
}

fn find(title_row: &[Data], name: impl AsRef<str>) -> Result<usize, AppErr> {
    title_row
        .iter()
        .position(|cell| cell == name.as_ref())
        .ok_or(format!("标题中找不到 {} 列", name.as_ref()).e_400())
}

pub fn i_analyze(i_title: &[Data]) -> Result<IContext, AppErr> {
    Ok(IContext {
        i_name: find(i_title, "商品名称")?,
        i_spec: find(i_title, "规格")?,
        i_p_easy: find(i_title, "7折价")?,
        i_p_normal: find(i_title, "5折价")?,
        i_p_hard: find(i_title, "3折价")?,
        i_p: find(i_title, "原价")?,
        i_p_score: find(i_title, "积分")?,
    })
}
pub fn s_analyze(s_title: &[Data]) -> Result<SContext, AppErr> {
    Ok(SContext {
        s_name: find(s_title, "姓名")?,
        s_no: find(s_title, "学号")?,
        s_d_level: find(s_title, "认定级别")?,
        s_second_school: s_title.iter().position(|cell| cell == "学院"),
        s_class: s_title.iter().position(|cell| cell == "班级"),
        s_sex: s_title.iter().position(|cell| cell == "性别"),
        s_credit: s_title.iter().position(|cell| cell == "余额"),
    })
}

pub async fn i_import(
    conn: &impl ConnectionTrait,
    i_ctx: &IContext,
    i_rows: impl Iterator<Item = &[Data]>,
) -> Result<usize, AppErr> {
    let i_start_id = item::Entity::next_pk(conn).await?;
    let i_data = i_rows
        .enumerate()
        .map(|(y, row)| {
            let y32 = y as i32;
            i_build_active_model(row, &i_ctx, y, Set(i_start_id + y32))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let i_len = i_data.len();
    if i_len > 0 {
        item::Entity::insert_many(i_data).exec(conn).await?;
    }
    Ok(i_len)
}

pub async fn s_import(
    conn: &impl ConnectionTrait,

    s_ctx: &SContext,
    s_rows: impl Iterator<Item = &[Data]>,
) -> Result<usize, AppErr> {
    let s_start_id = student::Entity::next_pk(conn).await?;
    let s_data = s_rows
        .enumerate()
        .map(|(y, row)| {
            let y32 = y as i32;
            s_build_active_model(row, &s_ctx, y, Set(s_start_id + y32))
        })
        .collect::<Result<Vec<_>, AppErr>>()?;

    let s_len = s_data.len();
    if s_len > 0 {
        student::Entity::insert_many(s_data).exec(conn).await?;
    }
    Ok(s_len)
}

fn string(row: &[Data], x: usize, y: usize) -> Result<ActiveValue<String>, AppErr> {
    row.get(x)
        .map(ToString::to_string)
        .map(Set)
        .ok_or(format!("第{}列 第{}列 数据缺失", x, y + 2).e_400())
}

fn string_null(row: &[Data], x: Option<usize>) -> ActiveValue<Option<String>> {
    Set(x.and_then(|i| row.get(i).map(ToString::to_string)))
}

fn decimal(row: &[Data], x: usize, y: usize) -> Result<ActiveValue<Decimal>, AppErr> {
    row.get(x)
        .map(ToString::to_string)
        .and_then(|s| Decimal::from_str(&s).ok())
        .map(Set)
        .ok_or(format!("第{}行 第{}列 数据缺失", y + 2, x + 1).e_400())
}
fn decimal_null(row: &[calamine::Data], x: Option<usize>) -> ActiveValue<Option<Decimal>> {
    Set(x
        .and_then(|i| row.get(i))
        .map(ToString::to_string)
        .and_then(|ref s| Decimal::from_str(s).ok()))
}
/// y 行号 供提示信息使用
/// y 会自动+2
/// id: 模型主键
pub fn i_build_active_model(
    row: &[Data],
    ctx: &IContext,
    y: usize,
    id: ActiveValue<i32>,
) -> Result<item::ActiveModel, AppErr> {
    Ok(item::ActiveModel {
        id,
        name: string(row, ctx.i_name, y)?,
        spec: string(row, ctx.i_spec, y)?,
        p_score: decimal(row, ctx.i_p_score, y)?,
        price: decimal(row, ctx.i_p, y)?,

        p_easy: decimal(row, ctx.i_p_easy, y)?,
        p_normal: decimal(row, ctx.i_p_normal, y)?,
        p_hard: decimal(row, ctx.i_p_hard, y)?,
        ..Default::default()
    })
}
/// y 行号 供提示信息使用
/// y会自动+2
pub fn s_build_active_model(
    row: &[Data],
    ctx: &SContext,
    y: usize,
    id: ActiveValue<i32>,
) -> Result<student::ActiveModel, AppErr> {
    Ok(student::ActiveModel {
        id,
        name: string(row, ctx.s_name, y)?,
        student_no: string(row, ctx.s_no, y)?,
        difficulty_level: string(row, ctx.s_d_level, y)?,
        secondary_school: string_null(row, ctx.s_second_school),
        class: string_null(row, ctx.s_class),
        sex: string_null(row, ctx.s_sex),
        balance: decimal_null(row, ctx.s_credit),
    })
}
