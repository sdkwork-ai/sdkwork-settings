-- SDKWork Settings 初始化 baseline (sqlite)
-- 应用处于初始化阶段:完整 DDL 位于此文件;migrations/ 保留用于 GA 后的变更。
-- 表前缀: stg_
-- 锚点表: stg_user_preference
-- 注意: SQLite 使用 TEXT 存储 JSON,使用 TEXT 存储 TIMESTAMPTZ(ISO-8601 字符串)

-- 用户偏好表
CREATE TABLE IF NOT EXISTS stg_user_preference (
    id INTEGER NOT NULL,
    tenant_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    namespace TEXT NOT NULL,
    preference_key TEXT NOT NULL,
    preference_value TEXT NOT NULL,
    value_type TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_by INTEGER,
    PRIMARY KEY (id),
    UNIQUE (tenant_id, user_id, namespace, preference_key),
    CHECK (value_type IN ('string', 'number', 'boolean', 'object', 'array', 'i18n'))
);

CREATE INDEX IF NOT EXISTS idx_stg_user_preference_tenant_user_ns
    ON stg_user_preference (tenant_id, user_id, namespace);

CREATE INDEX IF NOT EXISTS idx_stg_user_preference_updated_at
    ON stg_user_preference (updated_at);

-- 租户配置表
CREATE TABLE IF NOT EXISTS stg_tenant_config (
    id INTEGER NOT NULL,
    tenant_id TEXT NOT NULL,
    namespace TEXT NOT NULL,
    config_key TEXT NOT NULL,
    config_value TEXT NOT NULL,
    value_type TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_by INTEGER,
    PRIMARY KEY (id),
    UNIQUE (tenant_id, namespace, config_key),
    CHECK (value_type IN ('string', 'number', 'boolean', 'object', 'array', 'i18n'))
);

CREATE INDEX IF NOT EXISTS idx_stg_tenant_config_tenant_ns
    ON stg_tenant_config (tenant_id, namespace);

CREATE INDEX IF NOT EXISTS idx_stg_tenant_config_updated_at
    ON stg_tenant_config (updated_at);

-- 系统设置表
CREATE TABLE IF NOT EXISTS stg_system_setting (
    id INTEGER NOT NULL,
    namespace TEXT NOT NULL,
    setting_key TEXT NOT NULL,
    setting_value TEXT NOT NULL,
    value_type TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT 'global',
    scope_value TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_by INTEGER,
    PRIMARY KEY (id),
    UNIQUE (namespace, setting_key, scope, scope_value),
    CHECK (value_type IN ('string', 'number', 'boolean', 'object', 'array', 'i18n')),
    CHECK (scope IN ('global', 'region'))
);

CREATE INDEX IF NOT EXISTS idx_stg_system_setting_ns
    ON stg_system_setting (namespace);

CREATE INDEX IF NOT EXISTS idx_stg_system_setting_scope
    ON stg_system_setting (scope, scope_value);

-- 配置变更历史表
CREATE TABLE IF NOT EXISTS stg_config_revision (
    id INTEGER NOT NULL,
    tenant_id TEXT NOT NULL,
    config_type TEXT NOT NULL,
    config_id INTEGER NOT NULL,
    namespace TEXT NOT NULL,
    config_key TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    operation TEXT NOT NULL,
    operator_id INTEGER NOT NULL,
    operator_ip TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (id),
    CHECK (config_type IN ('user', 'tenant', 'system')),
    CHECK (operation IN ('create', 'update', 'delete'))
);

CREATE INDEX IF NOT EXISTS idx_stg_config_revision_tenant_type_id_time
    ON stg_config_revision (tenant_id, config_type, config_id, created_at);

CREATE INDEX IF NOT EXISTS idx_stg_config_revision_created_at
    ON stg_config_revision (created_at);
