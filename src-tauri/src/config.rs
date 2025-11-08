use rust_decimal::Decimal;
use rust_decimal::dec;
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{Arc, LazyLock, OnceLock},
};
use tokio::sync::RwLock;
use ts_rs::TS;
pub struct Config(Arc<RwLock<ConfigInner>>);

impl Deref for Config {
    type Target = Arc<RwLock<ConfigInner>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use crate::DATADIR;
const CONFIG_FILE: &str = "config.json";

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let conf = std::fs::read_to_string(DATADIR.join(CONFIG_FILE)).unwrap_or_default();
    let inner: ConfigInner = serde_json::from_str(&conf).unwrap_or_default();
    Config(Arc::new(RwLock::new(inner)))
});

impl Config {
    pub async fn reload() {
        let conf = tokio::fs::read_to_string(DATADIR.join(CONFIG_FILE))
            .await
            .unwrap_or_default();
        let inner: ConfigInner = serde_json::from_str(&conf).unwrap_or_default();
        let mut w = CONFIG.write().await;
        *w = inner;
    }

    pub async fn conf() -> ConfigInner {
        CONFIG.read().await.clone()
    }

    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = DATADIR.join(CONFIG_FILE);
        let content = serde_json::to_string_pretty(&*self.read().await)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }
}

impl Config {
    pub async fn export_path() -> Option<String> {
        CONFIG.read().await.export_config.path.clone()
    }

    pub async fn set_export_path(path: Option<String>) {
        let mut w = CONFIG.write().await;
        w.export_config.path = path;
    }

    pub async fn advance_export_path() -> Option<String> {
        CONFIG.read().await.export_config.advance_path.clone()
    }

    pub async fn set_advance_export_path(path: Option<String>) {
        let mut w = CONFIG.write().await;
        w.export_config.advance_path = path;
    }
    pub async fn balance_config() -> BalanceConfig {
        CONFIG.read().await.balance_config.clone()
    }

    pub async fn get_date_format() -> Option<String> {
        CONFIG.read().await.export_config.date_format.clone()
    }
}

impl Default for ConfigInner {
    fn default() -> Self {
        Self {
            export_config: ExportConfig {
                path: None,
                advance_path: None,
                template: Vec::new(),
                time_template: Vec::new(),
                date_format: None,
            },
            balance_config: BalanceConfig {
                active: true,
                tolerance: dec!(5.00),
                ignore_tolerance: true,
                pay_for_original_price: true,
                default_balance: DefaultBalance {
                    peaceful_balance: dec!(0.00),
                    easy_balance: dec!(70.00),
                    normal_balance: dec!(100.00),
                    hard_balance: dec!(150.00),
                },
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "config.ts")]
pub struct ConfigInner {
    pub export_config: ExportConfig,
    pub balance_config: BalanceConfig,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "config.ts")]
pub struct ExportConfig {
    /// csv导出路径
    pub path: Option<String>,
    /// xlsx导出路径
    pub advance_path: Option<String>,

    /// 导出模板
    /// 人员名字
    pub template: Vec<String>,
    /// 时间模板
    /// 值班时间段
    pub time_template: Vec<String>,
    ///  配置导出签名日期格式
    /// （导出签名上的日志由时间api返回，所以这对应chrono的日期配置
    /// 推荐配置为 "%Y %m %d"
    pub date_format: Option<String>,
}

use std::primitive::str;
/// 余额配置
#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "config.ts")]
pub struct BalanceConfig {
    /// 是否启用余额功能
    /// 默认为 true
    /// false 将不在进行任何有关余额初始化、计算的逻辑，并隐藏购买界面上的余额
    /// 余额为虚拟的额度，用处是限制每学期可购买的价值总额，每学期需要手动重置
    pub active: bool,

    /// 余额计算方式
    /// 默认为 true
    /// true 则按原价扣余额
    /// false 则按折后价扣余额
    pub pay_for_original_price: bool,

    /// 容错额度
    /// 默认为 5.00，与旧版一致
    /// 允许“负余额”，即允许余额小于0，但不能小于 -tolerance
    /// tolerance 应该是一个非负数
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub tolerance: Decimal,

    /// 是否忽略容错额度的限制
    /// 默认为 true
    /// 即便余额小于 -tolerance，也能继续添加商品
    /// true 允许继续添加商品
    /// false 不允许继续添加商品,直接返回http错误
    pub ignore_tolerance: bool,

    /// 各困难等级的 初始化额度
    pub default_balance: DefaultBalance,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "config.ts")]
pub struct DefaultBalance {
    /// “不困难" 的 初始化额度
    /// 默认为 0.00
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub peaceful_balance: Decimal,

    /// "一般困难" 的 初始化额度
    /// 默认为 70.00
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub easy_balance: Decimal,

    /// "困难" 的 初始化额度
    /// 默认为 100.00
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub normal_balance: Decimal,

    /// "特别困难" 的 初始化额度
    /// 默认为 150.00
    #[serde(with = "rust_decimal::serde::str")]
    #[ts(type = "string")]
    pub hard_balance: Decimal,
}
