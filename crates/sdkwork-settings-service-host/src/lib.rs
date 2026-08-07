//! SDKWork Settings 进程内服务容器。
//!
//! 持有数据库连接池,编排用户偏好、租户配置与系统配置的领域服务,
//! 负责输入校验、雪花 ID 生成、审计日志与事务执行。
//!
//! 校验与字符串工具复用 [`sdkwork_utils_rust`] 与 [`sdkwork_settings_contract`],
//! ID 生成复用 [`sdkwork_platform_id_service`] 的雪花算法。

mod error;

pub use error::ServiceError;

use chrono::{DateTime, Utc};
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_platform_id_service::SnowflakeIdGenerator;
use sdkwork_settings_contract::{
    SystemSetting, TenantConfig, UserPreference, validate_config_key, validate_namespace,
    validate_value_size,
};
use serde_json::Value;
use sqlx::FromRow;

/// 雪花 ID 节点号环境变量。
const SETTINGS_ID_NODE_ID_ENV: &str = "SDKWORK_SETTINGS_ID_NODE_ID";

/// Settings 服务容器。
pub struct SettingsServiceHost {
    pool: DatabasePool,
    id_generator: SnowflakeIdGenerator,
}

impl SettingsServiceHost {
    /// 基于已有连接池构造服务容器。
    ///
    /// 雪花节点号优先取自 `SDKWORK_SETTINGS_ID_NODE_ID`,缺省回退到 0
    /// (适合开发/测试;生产应使用数据库分配的稳定节点号)。
    pub fn new(pool: DatabasePool) -> Self {
        let node_id = std::env::var(SETTINGS_ID_NODE_ID_ENV)
            .ok()
            .and_then(|value| value.trim().parse::<u16>().ok())
            .unwrap_or(0);
        let id_generator = match SnowflakeIdGenerator::new(node_id) {
            Ok(generator) => generator,
            Err(error) => {
                tracing::warn!(?error, "snowflake init failed; falling back to node 0");
                SnowflakeIdGenerator::new(0).expect("snowflake node 0 must initialize")
            }
        };
        Self { pool, id_generator }
    }

    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    /// 生成下一个雪花 ID。
    fn next_id(&self) -> Result<i64, ServiceError> {
        self.id_generator
            .generate()
            .map_err(|error| ServiceError::Internal(format!("雪花 ID 生成失败: {error:?}")))
    }

    // -----------------------------------------------------------------
    // 用户偏好 (user preference)
    // -----------------------------------------------------------------

    /// 获取单个用户偏好。
    pub async fn get_user_preference(
        &self,
        tenant_id: &str,
        user_id: &str,
        namespace: &str,
        key: &str,
    ) -> Result<Option<UserPreference>, ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        tracing::debug!(tenant_id, user_id, namespace, key, "查询用户偏好");
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let row = sqlx::query_as::<_, UserPreferenceRow>(
                    "SELECT * FROM settings_user_preference \
                     WHERE tenant_id = $1 AND user_id = $2 AND namespace = $3 AND preference_key = $4",
                )
                .bind(tenant_id)
                .bind(user_id)
                .bind(namespace)
                .bind(key)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(UserPreference::from))
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 列出某用户在指定 namespace 下的全部偏好。
    pub async fn list_user_preferences(
        &self,
        tenant_id: &str,
        user_id: &str,
        namespace: &str,
    ) -> Result<Vec<UserPreference>, ServiceError> {
        validate_namespace(namespace)?;
        tracing::debug!(tenant_id, user_id, namespace, "列出用户偏好");
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let rows = sqlx::query_as::<_, UserPreferenceRow>(
                    "SELECT * FROM settings_user_preference \
                     WHERE tenant_id = $1 AND user_id = $2 AND namespace = $3",
                )
                .bind(tenant_id)
                .bind(user_id)
                .bind(namespace)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(UserPreference::from).collect())
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 更新(或新建)用户偏好,返回最终生效的记录。
    #[allow(clippy::too_many_arguments)]
    pub async fn update_user_preference(
        &self,
        tenant_id: &str,
        user_id: &str,
        namespace: &str,
        key: &str,
        value: Value,
        value_type: String,
        operator_id: &str,
    ) -> Result<UserPreference, ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        validate_value_size(&value)?;
        let id = self.next_id()?;
        let now = Utc::now();
        tracing::info!(
            tenant_id,
            user_id,
            namespace,
            key,
            operator_id,
            "更新用户偏好"
        );
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let row = sqlx::query_as::<_, UserPreferenceRow>(
                    "INSERT INTO settings_user_preference \
                     (id, tenant_id, user_id, namespace, preference_key, preference_value, value_type, created_at, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8) \
                     ON CONFLICT (tenant_id, user_id, namespace, preference_key) DO UPDATE SET \
                     preference_value = EXCLUDED.preference_value, \
                     value_type = EXCLUDED.value_type, \
                     updated_at = EXCLUDED.updated_at \
                     RETURNING *",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(user_id)
                .bind(namespace)
                .bind(key)
                .bind(value)
                .bind(&value_type)
                .bind(now)
                .fetch_one(pool)
                .await?;
                Ok(row.into())
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 删除用户偏好。`operator_id` 用于审计日志。
    pub async fn delete_user_preference(
        &self,
        tenant_id: &str,
        user_id: &str,
        namespace: &str,
        key: &str,
        operator_id: &str,
    ) -> Result<(), ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        tracing::info!(
            tenant_id,
            user_id,
            namespace,
            key,
            operator_id,
            "删除用户偏好"
        );
        let affected = match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                sqlx::query(
                    "DELETE FROM settings_user_preference \
                     WHERE tenant_id = $1 AND user_id = $2 AND namespace = $3 AND preference_key = $4",
                )
                .bind(tenant_id)
                .bind(user_id)
                .bind(namespace)
                .bind(key)
                .execute(pool)
                .await?
                .rows_affected()
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        };
        if affected == 0 {
            return Err(ServiceError::NotFound);
        }
        Ok(())
    }

    // -----------------------------------------------------------------
    // 租户配置 (tenant config)
    // -----------------------------------------------------------------

    /// 获取单个租户配置。
    pub async fn get_tenant_config(
        &self,
        tenant_id: &str,
        namespace: &str,
        key: &str,
    ) -> Result<Option<TenantConfig>, ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        tracing::debug!(tenant_id, namespace, key, "查询租户配置");
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let row = sqlx::query_as::<_, TenantConfigRow>(
                    "SELECT * FROM settings_tenant_config \
                     WHERE tenant_id = $1 AND namespace = $2 AND config_key = $3",
                )
                .bind(tenant_id)
                .bind(namespace)
                .bind(key)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(TenantConfig::from))
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 列出某租户在指定 namespace 下的全部配置。
    pub async fn list_tenant_configs(
        &self,
        tenant_id: &str,
        namespace: &str,
    ) -> Result<Vec<TenantConfig>, ServiceError> {
        validate_namespace(namespace)?;
        tracing::debug!(tenant_id, namespace, "列出租户配置");
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let rows = sqlx::query_as::<_, TenantConfigRow>(
                    "SELECT * FROM settings_tenant_config \
                     WHERE tenant_id = $1 AND namespace = $2",
                )
                .bind(tenant_id)
                .bind(namespace)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(TenantConfig::from).collect())
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 更新(或新建)租户配置。
    pub async fn update_tenant_config(
        &self,
        tenant_id: &str,
        namespace: &str,
        key: &str,
        value: Value,
        value_type: String,
        operator_id: &str,
    ) -> Result<TenantConfig, ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        validate_value_size(&value)?;
        let id = self.next_id()?;
        let now = Utc::now();
        tracing::info!(tenant_id, namespace, key, operator_id, "更新租户配置");
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let row = sqlx::query_as::<_, TenantConfigRow>(
                    "INSERT INTO settings_tenant_config \
                     (id, tenant_id, namespace, config_key, config_value, value_type, created_at, updated_at, created_by, updated_by) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8, $8) \
                     ON CONFLICT (tenant_id, namespace, config_key) DO UPDATE SET \
                     config_value = EXCLUDED.config_value, \
                     value_type = EXCLUDED.value_type, \
                     updated_at = EXCLUDED.updated_at, \
                     updated_by = EXCLUDED.updated_by \
                     RETURNING *",
                )
                .bind(id)
                .bind(tenant_id)
                .bind(namespace)
                .bind(key)
                .bind(value)
                .bind(&value_type)
                .bind(now)
                .bind(operator_id)
                .fetch_one(pool)
                .await?;
                Ok(row.into())
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 删除租户配置。
    pub async fn delete_tenant_config(
        &self,
        tenant_id: &str,
        namespace: &str,
        key: &str,
        operator_id: &str,
    ) -> Result<(), ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        tracing::info!(tenant_id, namespace, key, operator_id, "删除租户配置");
        let affected = match &self.pool {
            DatabasePool::Postgres(pool, _) => sqlx::query(
                "DELETE FROM settings_tenant_config \
                     WHERE tenant_id = $1 AND namespace = $2 AND config_key = $3",
            )
            .bind(tenant_id)
            .bind(namespace)
            .bind(key)
            .execute(pool)
            .await?
            .rows_affected(),
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        };
        if affected == 0 {
            return Err(ServiceError::NotFound);
        }
        Ok(())
    }

    // -----------------------------------------------------------------
    // 系统配置 (system setting)
    // -----------------------------------------------------------------

    /// 获取单个系统配置。
    pub async fn get_system_setting(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<SystemSetting>, ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        tracing::debug!(namespace, key, "查询系统配置");
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let row = sqlx::query_as::<_, SystemSettingRow>(
                    "SELECT * FROM settings_system_setting \
                     WHERE namespace = $1 AND setting_key = $2",
                )
                .bind(namespace)
                .bind(key)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(SystemSetting::from))
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 列出指定 namespace 下的全部系统配置。
    pub async fn list_system_settings(
        &self,
        namespace: &str,
    ) -> Result<Vec<SystemSetting>, ServiceError> {
        validate_namespace(namespace)?;
        tracing::debug!(namespace, "列出系统配置");
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let rows = sqlx::query_as::<_, SystemSettingRow>(
                    "SELECT * FROM settings_system_setting WHERE namespace = $1",
                )
                .bind(namespace)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(SystemSetting::from).collect())
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 更新(或新建)系统配置。
    #[allow(clippy::too_many_arguments)]
    pub async fn update_system_setting(
        &self,
        namespace: &str,
        key: &str,
        value: Value,
        value_type: String,
        scope: String,
        scope_value: Option<String>,
        operator_id: &str,
    ) -> Result<SystemSetting, ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        validate_value_size(&value)?;
        let id = self.next_id()?;
        let now = Utc::now();
        tracing::info!(namespace, key, scope, operator_id, "更新系统配置");
        match &self.pool {
            DatabasePool::Postgres(pool, _) => {
                let row = sqlx::query_as::<_, SystemSettingRow>(
                    "INSERT INTO settings_system_setting \
                     (id, namespace, setting_key, setting_value, value_type, scope, scope_value, created_at, updated_at, created_by, updated_by) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8, $9, $9) \
                     ON CONFLICT (namespace, setting_key) DO UPDATE SET \
                     setting_value = EXCLUDED.setting_value, \
                     value_type = EXCLUDED.value_type, \
                     scope = EXCLUDED.scope, \
                     scope_value = EXCLUDED.scope_value, \
                     updated_at = EXCLUDED.updated_at, \
                     updated_by = EXCLUDED.updated_by \
                     RETURNING *",
                )
                .bind(id)
                .bind(namespace)
                .bind(key)
                .bind(value)
                .bind(&value_type)
                .bind(&scope)
                .bind(scope_value)
                .bind(now)
                .bind(operator_id)
                .fetch_one(pool)
                .await?;
                Ok(row.into())
            }
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        }
    }

    /// 删除系统配置。
    pub async fn delete_system_setting(
        &self,
        namespace: &str,
        key: &str,
        operator_id: &str,
    ) -> Result<(), ServiceError> {
        validate_namespace(namespace)?;
        validate_config_key(key)?;
        tracing::info!(namespace, key, operator_id, "删除系统配置");
        let affected = match &self.pool {
            DatabasePool::Postgres(pool, _) => sqlx::query(
                "DELETE FROM settings_system_setting \
                     WHERE namespace = $1 AND setting_key = $2",
            )
            .bind(namespace)
            .bind(key)
            .execute(pool)
            .await?
            .rows_affected(),
            _ => {
                return Err(ServiceError::Database(format!(
                    "settings service requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"
                )));
            }
        };
        if affected == 0 {
            return Err(ServiceError::NotFound);
        }
        Ok(())
    }
}

// -----------------------------------------------------------------
// 内部行映射结构
// -----------------------------------------------------------------

#[derive(Debug, FromRow)]
struct UserPreferenceRow {
    id: i64,
    tenant_id: String,
    user_id: String,
    namespace: String,
    preference_key: String,
    preference_value: Value,
    value_type: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<String>,
    updated_by: Option<String>,
}

impl From<UserPreferenceRow> for UserPreference {
    fn from(row: UserPreferenceRow) -> Self {
        UserPreference {
            id: row.id,
            tenant_id: row.tenant_id,
            user_id: row.user_id,
            namespace: row.namespace,
            preference_key: row.preference_key,
            preference_value: row.preference_value,
            value_type: row.value_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
        }
    }
}

#[derive(Debug, FromRow)]
struct TenantConfigRow {
    id: i64,
    tenant_id: String,
    namespace: String,
    config_key: String,
    config_value: Value,
    value_type: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: String,
    updated_by: String,
}

impl From<TenantConfigRow> for TenantConfig {
    fn from(row: TenantConfigRow) -> Self {
        TenantConfig {
            id: row.id,
            tenant_id: row.tenant_id,
            namespace: row.namespace,
            config_key: row.config_key,
            config_value: row.config_value,
            value_type: row.value_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
        }
    }
}

#[derive(Debug, FromRow)]
struct SystemSettingRow {
    id: i64,
    namespace: String,
    setting_key: String,
    setting_value: Value,
    value_type: String,
    scope: String,
    scope_value: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: String,
    updated_by: String,
}

impl From<SystemSettingRow> for SystemSetting {
    fn from(row: SystemSettingRow) -> Self {
        SystemSetting {
            id: row.id,
            namespace: row.namespace,
            setting_key: row.setting_key,
            setting_value: row.setting_value,
            value_type: row.value_type,
            scope: row.scope,
            scope_value: row.scope_value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
        }
    }
}
