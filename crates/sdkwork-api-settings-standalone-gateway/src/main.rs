//! SDKWork Settings standalone 网关进程入口。
//!
//! standalone 和 cloud profile 下,Settings 应用面入口由同一个薄网关进程承载:
//! 1. 基础设施探测(`/healthz`、`/readyz`、`/livez`、`/metrics`)
//! 2. app-api 业务路由(`/settings/v1/app-api/*`)
//! 3. backend-api 业务路由(`/settings/v1/backend-api/*`)
//!
//! 服务构造(数据库引导、连接池、服务宿主、应用状态)与业务路由装配全部由
//! `sdkwork-api-settings-assembly` 拥有;本 crate 只负责进程启动、环境加载、
//! 基础设施探测和监听(API_ASSEMBLY_SPEC §6.1)。
//!
//! # 环境变量
//!
//! - `SDKWORK_DATABASE_*`: 统一的 PostgreSQL 连接与 schema 配置
//! - `SDKWORK_SETTINGS_APPLICATION_PUBLIC_INGRESS_BIND`: 应用公开入口监听地址(默认 `0.0.0.0:8080`)

use sdkwork_api_settings_assembly::assemble_api_router_from_env;
use sdkwork_web_bootstrap::ApiModuleRegistry;
use sdkwork_settings_web_bootstrap::{
    mount_settings_infra_routes, settings_service_router_config,
};
use sdkwork_web_bootstrap::serve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!(target: "sdkwork_settings::standalone_gateway", "starting sdkwork-api-settings-standalone-gateway");

    // 装配路由: assembly owns service construction and business routes; listener owns infra routes.
    let mut module_registry = ApiModuleRegistry::new();
    module_registry.add_module(assemble_api_router_from_env()
        .await
        .map_err(|e| format!("Settings 应用装配失败: {e}"))?);
    let assembly = module_registry.try_compose("SDKWork Settings API")?;
    let router = mount_settings_infra_routes(assembly.router, settings_service_router_config());
    tracing::info!(target: "sdkwork_settings::standalone_gateway", "router assembled through gateway assembly");

    // 启动 HTTP 服务
    let bind = std::env::var("SDKWORK_SETTINGS_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_owned());
    tracing::info!(target: "sdkwork_settings::standalone_gateway", bind = %bind, "listening");

    let addr: std::net::SocketAddr = bind.parse()?;
    serve(router, addr).await?;

    tracing::info!(target: "sdkwork_settings::standalone_gateway", "gateway shutdown complete");
    Ok(())
}
