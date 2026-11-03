# SDKWork Settings

repository-kind: application

SDKWork Settings — configuration center providing multi-tenant, multi-user, multi-language configuration management with multi-architecture integration support.

## What This Is

A configuration center that lets different SDKWork applications (PC, H5, Flutter, mini program, native Android/iOS/Harmony, backend services) integrate settings, preferences, and tenant config through a unified API and SDK surface instead of each application reimplementing its own settings layer.

## Documentation Canon

- [docs/README.md](docs/README.md) — documentation index and audience routing
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md) — product requirements
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md) — technical architecture

## Getting Started

```bash
# Install dependencies
pnpm install

# Start development (default PostgreSQL, standalone.development)
pnpm dev

# Build
pnpm build

# Verify
pnpm verify
```

## Framework Integration

This application integrates the following SDKWork frameworks per `sdkwork-specs`:

- `sdkwork-web-framework` — mandatory HTTP API framework
- `sdkwork-database` — database lifecycle framework
- `sdkwork-utils` — cross-language utility library
- `sdkwork-appbase` — platform ID service and PC React runtime
- `sdkwork-iam` — authentication, authorization, application bootstrap
- `sdkwork-drive` — file upload (avatar, logo, config import, attachments)
- `sdkwork-discovery` — deferred (no RPC services in Phase 1)

## Frontend Composition

The PC settings surface is built from the shared SDKWork Appbase settings
components:

- `@sdkwork/settings-core` — catalog, layered values, validation, save modes,
  slots, i18n, and the appType adapter matrix.
- `@sdkwork/settings-pc-react` — `SdkworkSettingsProvider`,
  `SdkworkSettingsCenter`, accessible controls, and slot rendering.

The application root supplies the runtime, a `SdkworkSettingsStorePort`, theme
synchronization, and action handlers. It does not fork settings navigation,
validation, dirty tracking, or save semantics. See
[apps/sdkwork-settings-pc/README.md](apps/sdkwork-settings-pc/README.md).

## Standards

This repository follows `../sdkwork-specs/` global standards by relative path. See [AGENTS.md](AGENTS.md) for the task-to-spec loading matrix.
