use rust_decimal::Decimal;
use rust_decimal::dec;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::primitive::str;
use std::{ops::Deref, sync::Arc};
use tauri::Manager;
use tauri::Runtime;
use tokio::sync::RwLock;
use ts_rs::TS;

pub const CONFIG_FILE_NAME: &str = "chrm-rev.config.json";
pub const DB_FILE_NAME: &str = "chrm-rev.sqlite.db";

#[derive(Clone, Default)]
pub struct Config(Arc<RwLock<ConfigInner>>);

impl Config {
    pub fn new(arc: ConfigInner) -> Self {
        Self(Arc::new(RwLock::new(arc)))
    }
    pub fn clone_(&self) -> Self {
        self.clone()
    }
    pub async fn export_path(&self) -> PathBuf {
        self.0.read().await.export_path.clone()
    }
    pub async fn legacy_export_format(&self) -> bool {
        self.0.read().await.legacy_export_format
    }
    pub async fn user_template(&self) -> Vec<String> {
        self.0.read().await.user_template.clone()
    }
    pub async fn time_template(&self) -> Vec<String> {
        self.0.read().await.time_template.clone()
    }
    pub async fn balance_config(&self) -> BalanceConfig {
        self.0.read().await.balance_config
    }
    pub async fn default_balance(&self) -> DefaultBalance {
        self.0.read().await.balance_config.default_balance
    }
}

impl Deref for Config {
    type Target = Arc<RwLock<ConfigInner>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "config_v2.ts")]
pub struct ConfigInner {
    /// 导出路径
    pub export_path: PathBuf,
    pub legacy_export_format: bool,
    /// 导出模板
    /// 人员名字
    pub user_template: Vec<String>,
    /// 时间模板
    /// 值班时间段
    pub time_template: Vec<String>,

    /// 余额配置
    pub balance_config: BalanceConfig,
}

impl ConfigInner {
    pub async fn save<RT: Runtime>(
        &self,
        app_handle: &tauri::AppHandle<RT>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data_dir = app_handle.path().app_data_dir()?;
        let conf_path = data_dir.join(CONFIG_FILE_NAME);
        let contents = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&conf_path, contents).await?;
        Ok(())
    }
}

impl Default for ConfigInner {
    fn default() -> Self {
        Self {
            export_path: PathBuf::new(),
            legacy_export_format: true,
            user_template: Vec::new(),
            time_template: vec![
                "09:30-11:00".into(),
                "11:00-12:30".into(),
                "15:00-17:00".into(),
                "17:00-18:30".into(),
                "19:30-21:00".into(),
            ],
            balance_config: BalanceConfig {
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

/// 余额配置
#[derive(Serialize, Deserialize, Clone, Copy, TS)]
#[ts(export, export_to = "config_v2.ts")]
pub struct BalanceConfig {
    /// 余额计算方式
    /// 默认为 true
    /// true 则按原价扣余额
    /// false 则按折后价扣余额
    pub pay_for_original_price: bool,

    /// 各困难等级的 初始化额度
    pub default_balance: DefaultBalance,
}

#[derive(Serialize, Deserialize, Clone, Copy, TS)]
#[ts(export, export_to = "config_v2.ts")]
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
