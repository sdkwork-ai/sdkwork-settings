//! Gateway bootstrap for sdkwork-settings.
//!
//! The assembly exports the indivisible `ApiAssemblyContribution` contract
//! (API_ASSEMBLY_SPEC.md §4): the executable business router, the combined
//! route manifest inventory, the derived OpenAPI document, the permission
//! catalog, domain context injectors, and the readiness check.
//!
//! The assembly owns Settings service construction (database bootstrap, pool,
//! service host, app state); the thin standalone gateway consumes
//! `assemble_api_router_from_env` and projects `.router`
//! (API_ASSEMBLY_SPEC §6.1).

use std::sync::Arc;

use sdkwork_settings_database_host::bootstrap_settings_database_from_env;
use sdkwork_settings_service_host::SettingsServiceHost;
use sdkwork_settings_web_bootstrap::{
    SettingsAppState, create_settings_router, wrap_settings_router_with_framework,
};
use sdkwork_web_contract::{HttpMethod, HttpRoute};
use sdkwork_web_core::HttpRouteManifest;

pub use sdkwork_web_bootstrap::ApiAssemblyContribution;

pub type ApiAssembly = ApiAssemblyContribution;

/// app-api route inventory, aligned with
/// `apis/app-api/settings/sdkwork-settings-app-api.openapi.json`.
const APP_API_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/settings/v1/app-api/preferences",
        "settings",
        "preferences.list",
    )
    .with_required_permission("iam:self"),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/settings/v1/app-api/preferences/{namespace}",
        "settings",
        "preferences.entries.list",
    )
    .with_required_permission("iam:self"),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/settings/v1/app-api/preferences/{namespace}/{key}",
        "settings",
        "preferences.entries.retrieve",
    )
    .with_required_permission("iam:self"),
    HttpRoute::dual_token(
        HttpMethod::Put,
        "/settings/v1/app-api/preferences/{namespace}/{key}",
        "settings",
        "preferences.entries.update",
    )
    .with_required_permission("iam:self"),
    HttpRoute::dual_token(
        HttpMethod::Delete,
        "/settings/v1/app-api/preferences/{namespace}/{key}",
        "settings",
        "preferences.entries.delete",
    )
    .with_required_permission("iam:self"),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/settings/v1/app-api/preferences:batchUpdate",
        "settings",
        "preferences.batchUpdate",
    )
    .with_required_permission("iam:self"),
];

/// backend-api route inventory, aligned with
/// `apis/backend-api/settings/sdkwork-settings-backend-api.openapi.json`.
const BACKEND_API_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/settings/v1/backend-api/tenant-configs",
        "settings",
        "tenantConfigs.list",
    )
    .with_required_permission("iam:tenant:admin"),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/settings/v1/backend-api/tenant-configs/{namespace}/{key}",
        "settings",
        "tenantConfigs.retrieve",
    )
    .with_required_permission("iam:tenant:admin"),
    HttpRoute::dual_token(
        HttpMethod::Put,
        "/settings/v1/backend-api/tenant-configs/{namespace}/{key}",
        "settings",
        "tenantConfigs.update",
    )
    .with_required_permission("iam:tenant:admin"),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/settings/v1/backend-api/system-settings",
        "settings",
        "systemSettings.list",
    )
    .with_required_permission("iam:system:admin"),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/settings/v1/backend-api/system-settings/{namespace}/{key}",
        "settings",
        "systemSettings.retrieve",
    )
    .with_required_permission("iam:system:admin"),
    HttpRoute::dual_token(
        HttpMethod::Put,
        "/settings/v1/backend-api/system-settings/{namespace}/{key}",
        "settings",
        "systemSettings.update",
    )
    .with_required_permission("iam:system:admin"),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/settings/v1/backend-api/revisions",
        "settings",
        "revisions.list",
    )
    .with_required_permission("iam:tenant:admin"),
];

fn combined_route_manifest() -> HttpRouteManifest {
    let mut routes = Vec::new();
    routes.extend_from_slice(APP_API_ROUTES);
    routes.extend_from_slice(BACKEND_API_ROUTES);
    HttpRouteManifest::from_owned_routes(routes)
}

pub async fn assemble_api_router(state: SettingsAppState) -> Result<ApiAssembly, String> {
    let router = create_settings_router(state);
    let router = wrap_settings_router_with_framework(router).await;
    ApiAssemblyContribution::from_manifest(
        "sdkwork-settings",
        "SDKWork Settings API",
        router,
        combined_route_manifest(),
        Vec::new(),
        Arc::new(sdkwork_web_bootstrap::AlwaysReady),
    )
}

/// Boots the Settings service (database bootstrap, pool, service host, app
/// state) from `SDKWORK_DATABASE_*` environment, then assembles the business
/// router contribution. The thin standalone gateway calls only this entry and
/// projects `.router` (API_ASSEMBLY_SPEC §6.1).
pub async fn assemble_api_router_from_env() -> Result<ApiAssembly, String> {
    let db_host = bootstrap_settings_database_from_env()
        .await
        .map_err(|error| format!("Settings 数据库引导失败: {error}"))?;
    let host = Arc::new(SettingsServiceHost::new(db_host.pool().clone()));
    let state = SettingsAppState::new(host);
    assemble_api_router(state).await
}
