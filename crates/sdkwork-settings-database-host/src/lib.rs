//! SDKWork Settings 数据库生命周期引导模块。
//!
//! 接入 `sdkwork-database` 框架,负责加载 schema 清单、初始化与迁移。
//! 参考 `sdkwork-im-database-host` 模式,环境变量前缀使用 `SETTINGS`
//! (例如 `SDKWORK_SETTINGS_APP_ROOT`)。

use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_lifecycle::{LifecycleOrchestrator, lifecycle_options_from_env};
use sdkwork_database_spi::{DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule};
use sdkwork_database_sqlx::{DatabasePool, create_pool_from_config};

/// Settings 数据库宿主,持有连接池与数据库模块清单。
pub struct SettingsDatabaseHost {
    pool: DatabasePool,
    module: Arc<DefaultDatabaseModule>,
}

impl SettingsDatabaseHost {
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn module(&self) -> Arc<DefaultDatabaseModule> {
        self.module.clone()
    }
}

/// 基于已有连接池引导 Settings 数据库生命周期。
///
/// 读取应用根目录下的数据库清单,执行 init 与(可选)迁移。
pub async fn bootstrap_settings_database(
    pool: DatabasePool,
) -> Result<SettingsDatabaseHost, String> {
    let app_root = resolve_app_root();
    let module = Arc::new(
        DefaultDatabaseModule::from_app_root(&app_root)
            .map_err(|error| format!("load Settings database module failed: {error}"))?,
    );
    let manifest = DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read Settings database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("SETTINGS", &manifest);
    let orchestrator = LifecycleOrchestrator::new(pool.clone(), module.clone())
        .with_applied_by("sdkwork-settings");

    orchestrator
        .init()
        .await
        .map_err(|error| format!("Settings database init failed: {error}"))?;

    if options.auto_migrate {
        orchestrator
            .migrate()
            .await
            .map_err(|error| format!("Settings database migrate failed: {error}"))?;
    }

    Ok(SettingsDatabaseHost { pool, module })
}

/// 从环境变量引导 Settings 数据库(读取统一的 `SDKWORK_DATABASE_*` 配置)。
pub async fn bootstrap_settings_database_from_env() -> Result<SettingsDatabaseHost, String> {
    let _ = dotenvy::dotenv();
    let config = DatabaseConfig::from_env("SETTINGS")
        .map_err(|error| format!("read Settings database config failed: {error}"))?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create Settings database pool failed: {error}"))?;
    bootstrap_settings_database(pool).await
}

/// 解析 Settings 应用根目录。
///
/// 优先使用 `SDKWORK_SETTINGS_APP_ROOT` 环境变量,否则回退到本 crate
/// 所在 workspace 的根目录。
fn resolve_app_root() -> PathBuf {
    std::env::var("SDKWORK_SETTINGS_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        })
}
