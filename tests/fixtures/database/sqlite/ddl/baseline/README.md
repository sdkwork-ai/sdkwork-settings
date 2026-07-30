# Baseline DDL (SQLite)

SQLite 初始化 baseline DDL。

当前文件:
- `0001_settings_baseline.sql`: Settings 配置中心初始 schema(4 张表 + 索引 + 约束)

注意: SQLite 使用 TEXT 存储 JSON 和时间戳(ISO-8601 字符串),BIGINT 使用 INTEGER。
