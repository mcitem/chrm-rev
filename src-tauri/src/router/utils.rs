//! 和router本身相关的工具

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub(crate) mod export;
pub(crate) mod import;

#[derive(TS, Deserialize, Debug)]
#[ts(export, export_to = "./utils.ts")]
#[serde(rename_all = "lowercase")]
pub enum Order {
    Asc,
    Desc,
}

impl Into<sea_orm::Order> for Order {
    fn into(self) -> sea_orm::Order {
        match self {
            Order::Asc => sea_orm::Order::Asc,
            Order::Desc => sea_orm::Order::Desc,
        }
    }
}

#[derive(TS, Deserialize, Debug)]
#[ts(export, export_to = "./utils.ts")]
/// 不支持深层对象
///
/// https://github.com/jplatte/serde_html_form/issues/25
pub struct OrderPagination {
    #[ts(type = "number")]
    pub page: u64,
    #[ts(type = "number")]
    pub per_page: u64,
    pub order_by_key: Option<String>,
    pub order_by_type: Option<Order>,
    pub q: Option<String>,
}

// #[derive(TS, Deserialize, Debug)]
// #[ts(export, export_to = "./utils.ts")]
// pub(crate) struct PaginateParams {
//     #[ts(type = "number")]
//     /// 第几页 传入要求为从1开始
//     pub page: u64,
//     #[ts(type = "number")]
//     /// 页面大小 (page_size) (每页的数量)
//     pub per_page: u64,
// }

#[derive(TS, Serialize)]
#[ts(export, export_to = "./utils.ts")]
pub(crate) struct PaginateData<T> {
    #[ts(type = "number")]
    /// 参数返回
    pub page: u64,
    #[ts(type = "number")]
    /// 参数返回
    pub per_page: u64,

    #[ts(type = "number")]
    /// 总item数量
    pub length: u64,

    pub data: T,
}
