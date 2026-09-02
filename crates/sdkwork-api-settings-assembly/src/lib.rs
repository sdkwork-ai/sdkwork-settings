//! API assembly for sdkwork-settings.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM

mod bootstrap;
mod generated;

pub use bootstrap::{assemble_api_router, ApiAssembly, ApiAssemblyContribution, assemble_api_router_from_env, web_module};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
