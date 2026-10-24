use std::collections::HashSet;

use sea_orm::{IntoActiveModel, TransactionTrait};
use serde_json::{Map, Value};

use crate::router::prelude::*;
use crate::utils::NextPrimaryKey;

pub fn router() -> Router<AppState> {
    Router::new().route("/save", put(save))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    #[serde(default)]
    action: Action,
    kind: String,
    id: i32,
    before: Map<String, Value>,
    after: Map<String, Value>,
}

#[derive(Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Action {
    #[default]
    Update,
    Create,
    Delete,
}

fn invalid(message: impl Into<String>) -> BizErrKind {
    BizErrKind::InvalidEdit(message.into())
}

fn fields(kind: &str) -> Result<&'static [&'static str], BizErrKind> {
    match kind {
        "student" => Ok(&[
            "name",
            "student_no",
            "difficulty_level",
            "secondary_school",
            "class",
            "sex",
            "major",
            "balance",
        ]),
        "item" => Ok(&[
            "name", "spec", "price", "p_easy", "p_normal", "p_hard", "p_score",
        ]),
        _ => Err(invalid("无效的资料类型")),
    }
}

fn merge_edit(current: &mut Value, edit: &Edit) -> Result<(), BizErrKind> {
    let fields = fields(&edit.kind)?;
    if edit.after.is_empty() || edit.before.len() != edit.after.len() {
        return Err(invalid("修改字段不完整"));
    }
    for (key, value) in &edit.after {
        if !fields.contains(&key.as_str()) || !edit.before.contains_key(key) {
            return Err(invalid(format!("不允许修改字段：{key}")));
        }
        if current.get(key) != edit.before.get(key) {
            return Err(invalid(format!(
                "编号 {} 的 {} 已变化，请刷新后重新编辑",
                edit.id, key
            )));
        }
        let optional = ["secondary_school", "class", "sex", "major"].contains(&key.as_str());
        if optional && value.is_null() {
            current[key] = Value::Null;
            continue;
        }
        let text = value.as_str().ok_or_else(|| invalid("字段必须为文本"))?;
        if ["name", "student_no"].contains(&key.as_str()) && text.trim().is_empty() {
            return Err(invalid("姓名、商品名称和学号不能为空"));
        }
        if key == "difficulty_level" && !["不困难", "一般困难", "困难", "特别困难"].contains(&text)
        {
            return Err(invalid("请选择有效的困难级别"));
        }
        if key == "balance" || key == "price" || key.starts_with("p_") {
            let amount = Decimal::from_str(text).map_err(|_| invalid("金额格式无效"))?;
            if amount.scale() > 2
                || amount.abs() > Decimal::new(9999999999, 2)
                || (key != "balance" && amount < Decimal::ZERO)
            {
                return Err(invalid(
                    "金额最多两位小数且绝对值不超过 99999999.99，商品价格不能为负数",
                ));
            }
        }
        current[key] = value.clone();
    }
    Ok(())
}

fn create_value(edit: &Edit, id: i32) -> Result<Value, BizErrKind> {
    let required = fields(&edit.kind)?;
    if edit.id >= 0
        || !edit.before.is_empty()
        || edit.after.len() != required.len()
        || required.iter().any(|key| !edit.after.contains_key(*key))
    {
        return Err(invalid("新增行需包含全部资料，编号由系统分配"));
    }
    let mut value = Value::Object(edit.after.clone());
    merge_edit(
        &mut value,
        &Edit {
            action: Action::Update,
            kind: edit.kind.clone(),
            id,
            before: edit.after.clone(),
            after: edit.after.clone(),
        },
    )?;
    value["id"] = serde_json::json!(id);
    Ok(value)
}

fn check_delete(current: Value, edit: &Edit) -> Result<(), BizErrKind> {
    let required = fields(&edit.kind)?;
    if !edit.after.is_empty()
        || edit.before.len() != required.len()
        || required
            .iter()
            .any(|key| edit.before.get(*key) != current.get(*key))
    {
        return Err(invalid(format!(
            "编号 {} 的资料已变化或删除资料不完整，请刷新后重试",
            edit.id
        )));
    }
    Ok(())
}

async fn save(
    State(AppState { db, config }): State<AppState>,
    Json(mut edits): Json<Vec<Edit>>,
) -> Result<AppResponse<Value>, BizErrKind> {
    if edits.is_empty() || edits.len() > 1000 {
        return Err(invalid("每次请保存 1 至 1000 行修改"));
    }
    let _operation = config.operation().await;
    let tx = db.begin().await?;
    if record::Entity::find().one(&tx).await?.is_some() {
        return Err(invalid(
            "首页还有购买记录，不允许管理，请先导出并清空购买记录。",
        ));
    }
    // Allocate new IDs before deletions so a new row never takes the ID of a row deleted in this batch.
    edits.sort_by_key(|edit| edit.action != Action::Create);
    let mut created = Vec::new();
    let mut seen = HashSet::new();
    for edit in &edits {
        if !seen.insert((&edit.kind, edit.id, edit.action == Action::Create)) {
            return Err(invalid("同一行不能重复提交"));
        }
        match edit.kind.as_str() {
            "student" => {
                if edit.action == Action::Create {
                    let id = student::Entity::next_pk(&tx).await?;
                    let model: student::Model = serde_json::from_value(create_value(edit, id)?)
                        .map_err(|_| invalid("学生资料格式无效"))?;
                    if student::Entity::find()
                        .filter(student::Column::StudentNo.eq(&model.student_no))
                        .one(&tx)
                        .await?
                        .is_some()
                    {
                        return Err(invalid(format!(
                            "学号 {} 已被其他学生使用",
                            model.student_no
                        )));
                    }
                    model.into_active_model().insert(&tx).await?;
                    created.push(
                        serde_json::json!({"kind": edit.kind, "temporary_id": edit.id, "id": id}),
                    );
                    continue;
                }
                let model = student::Entity::find_by_id(edit.id)
                    .one(&tx)
                    .await?
                    .ok_or(BizErrKind::StuNotFound)?;
                if edit.action == Action::Delete {
                    check_delete(
                        serde_json::to_value(&model).map_err(|_| invalid("读取学生失败"))?,
                        edit,
                    )?;
                    if record::Entity::find()
                        .filter(record::Column::StudentId.eq(edit.id))
                        .one(&tx)
                        .await?
                        .is_some()
                    {
                        return Err(invalid(format!(
                            "学生编号 {} 有购买记录，不能直接删除",
                            edit.id
                        )));
                    }
                    model.into_active_model().delete(&tx).await?;
                    continue;
                }
                let mut value =
                    serde_json::to_value(&model).map_err(|_| invalid("读取学生失败"))?;
                merge_edit(&mut value, edit)?;
                let updated: student::Model =
                    serde_json::from_value(value).map_err(|_| invalid("学生资料格式无效"))?;
                if edit.after.contains_key("student_no")
                    && student::Entity::find()
                        .filter(student::Column::StudentNo.eq(&updated.student_no))
                        .filter(student::Column::Id.ne(edit.id))
                        .one(&tx)
                        .await?
                        .is_some()
                {
                    return Err(invalid(format!(
                        "学号 {} 已被其他学生使用",
                        updated.student_no
                    )));
                }
                // Update only submitted fields. In particular, profile edits must not overwrite balances.
                let mut active = model.into_active_model();
                let changed = updated.into_active_model();
                for key in edit.after.keys() {
                    let column = student::Column::from_str(key)?;
                    active.set(column, changed.get(column).unwrap());
                }
                active.update(&tx).await?;
            }
            "item" => {
                if edit.action == Action::Create {
                    let id = item::Entity::next_pk(&tx).await?;
                    let model: item::Model = serde_json::from_value(create_value(edit, id)?)
                        .map_err(|_| invalid("商品资料格式无效"))?;
                    model.into_active_model().insert(&tx).await?;
                    created.push(
                        serde_json::json!({"kind": edit.kind, "temporary_id": edit.id, "id": id}),
                    );
                    continue;
                }
                let model = item::Entity::find_by_id(edit.id)
                    .one(&tx)
                    .await?
                    .ok_or(BizErrKind::ItemNotFound)?;
                if edit.action == Action::Delete {
                    check_delete(
                        serde_json::to_value(&model).map_err(|_| invalid("读取商品失败"))?,
                        edit,
                    )?;
                    if record::Entity::find()
                        .filter(record::Column::ItemId.eq(edit.id))
                        .one(&tx)
                        .await?
                        .is_some()
                    {
                        return Err(invalid(format!(
                            "商品编号 {} 有购买记录，不能直接删除",
                            edit.id
                        )));
                    }
                    model.into_active_model().delete(&tx).await?;
                    continue;
                }
                let mut value =
                    serde_json::to_value(&model).map_err(|_| invalid("读取商品失败"))?;
                merge_edit(&mut value, edit)?;
                let updated: item::Model =
                    serde_json::from_value(value).map_err(|_| invalid("商品资料格式无效"))?;
                let mut active = model.into_active_model();
                let changed = updated.into_active_model();
                for key in edit.after.keys() {
                    let column = item::Column::from_str(key)?;
                    active.set(column, changed.get(column).unwrap());
                }
                active.update(&tx).await?;
            }
            _ => return Err(invalid("无效的资料类型")),
        }
    }
    tx.commit().await?;
    ok!({ "saved": edits.len(), "created": created })
}
