# SDKWork Settings Technical Architecture

Status: active
Owner: SDKWork Architecture
Updated: 2026-07-08
Specs: `../sdkwork-specs/APPLICATION_LAYERED_ARCHITECTURE_SPEC.md`, `../sdkwork-specs/APPLICATION_GATEWAY_SPEC.md`, `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`, `../sdkwork-specs/WEB_FRAMEWORK_SPEC.md`, `../sdkwork-specs/API_SPEC.md`, `../sdkwork-specs/DATABASE_SPEC.md`, `../sdkwork-specs/IAM_SPEC.md`

## 1. Architecture Overview

SDKWork Settings is the configuration center for multi-tenant, multi-user, and multi-locale application settings. It exposes user-facing `app-api` operations for preferences and operator-facing `backend-api` operations for tenant and system configuration.

```text
Client applications
  PC React, H5, Flutter, Mini Program, Android, iOS, Harmony
        |
        | SDKWork composed SDKs
        v
sdkwork-settings-app-sdk / sdkwork-settings-backend-sdk
        |
        | HTTP, SdkWorkApiResponse envelope
        v
application.public-ingress
  sdkwork-api-settings-standalone-gateway
        |
        | sdkwork-api-settings-assembly
        v
sdkwork-settings-web-bootstrap
  sdkwork-web-framework, IAM request context, infra routes
        |
        +-- sdkwork-routes-settings-app-api
        +-- sdkwork-routes-settings-backend-api
        |
        v
sdkwork-settings-service-host
        |
        +-- sdkwork-settings-contract
        +-- sdkwork-settings-database-host
        +-- sdkwork-iam
        +-- sdkwork-drive
        +-- sdkwork-utils-rust
```

`sdkwork-api-settings-standalone-gateway` is the active Settings application-plane ingress. `sdkwork-api-settings-assembly` owns route composition. Route crates remain HTTP adapters, not listener processes.

Retired `*-api-server` listeners are not part of the active architecture.

## 2. Technology Choices

| Concern | Choice |
| --- | --- |
| Backend language | Rust |
| HTTP framework | `sdkwork-web-framework` on Axum |
| API envelope | `SdkWorkApiResponse` success envelope and `ProblemDetail` errors |
| Application ingress | `sdkwork-api-settings-standalone-gateway` |
| Route composition | `sdkwork-api-settings-assembly` |
| Database | PostgreSQL by default, SQLite for supported local/runtime targets |
| Frontend | React, Vite, TypeScript |
| Authentication and authorization | `sdkwork-iam` and `sdkwork-iam-web-adapter` |
| Upload capability | `sdkwork-drive` uploader integration |
| Shared utilities | `sdkwork-utils-rust` |

## 3. System Boundaries And Modules

| Layer | Owner | Responsibility |
| --- | --- | --- |
| API route adapters | `sdkwork-routes-settings-app-api`, `sdkwork-routes-settings-backend-api` | HTTP decoding, request context extraction, response mapping, route manifests |
| Gateway assembly | `sdkwork-api-settings-assembly` | Business route composition and application-plane router exports |
| Application ingress | `sdkwork-api-settings-standalone-gateway` | Listener startup, topology bind env, process-level infra routes |
| Framework bootstrap | `sdkwork-settings-web-bootstrap` | SDKWork web-framework wrapping, IAM request context, business router construction |
| Service host | `sdkwork-settings-service-host` | In-process service container, repository and provider wiring |
| Domain contract | `sdkwork-settings-contract` | Domain DTOs, value objects, commands, results, and typed errors |
| Persistence | `sdkwork-settings-database-host` | Database lifecycle, schema registration, repository wiring |

Layering rules:

- Route crates do not own business rules or SQL.
- Service host wires dependencies but does not start HTTP listeners.
- Application ingress does not hand-merge route crates; it calls the gateway assembly entrypoint.
- Generated SDK families are consumed through composed package surfaces, not raw generated transport names.

## 4. Directory And Package Layout

```text
apis/                         authored OpenAPI contracts
apps/sdkwork-settings-pc/      PC browser and desktop renderer root
crates/
  sdkwork-settings-contract/
  sdkwork-settings-database-host/
  sdkwork-settings-service-host/
  sdkwork-settings-web-bootstrap/
  sdkwork-api-settings-assembly/
  sdkwork-api-settings-standalone-gateway/
  sdkwork-routes-settings-app-api/
  sdkwork-routes-settings-backend-api/
etc/topology/              topology profiles and env templates
deployments/                   deployment manifests
sdks/                          SDK families and generated SDK artifacts
specs/                         repository/application machine contracts
```

`services/` is reserved for future non-HTTP workers or schedulers. It does not contain the active application HTTP ingress.

## 5. API, SDK, And Data Ownership

### app-api

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/settings/v1/app-api/preferences` | List current user preferences |
| `GET` | `/settings/v1/app-api/preferences/{namespace}` | Get preferences in one namespace |
| `GET` | `/settings/v1/app-api/preferences/{namespace}/{key}` | Get one preference |
| `PUT` | `/settings/v1/app-api/preferences/{namespace}/{key}` | Update one preference |
| `DELETE` | `/settings/v1/app-api/preferences/{namespace}/{key}` | Reset one preference to inherited default |
| `POST` | `/settings/v1/app-api/preferences:batchUpdate` | Batch update preferences |

### backend-api

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/settings/v1/backend-api/tenant-configs` | List tenant configuration |
| `GET` | `/settings/v1/backend-api/tenant-configs/{namespace}/{key}` | Get one tenant configuration value |
| `PUT` | `/settings/v1/backend-api/tenant-configs/{namespace}/{key}` | Update one tenant configuration value |
| `GET` | `/settings/v1/backend-api/system-settings` | List system settings |
| `GET` | `/settings/v1/backend-api/system-settings/{namespace}/{key}` | Get one system setting |
| `PUT` | `/settings/v1/backend-api/system-settings/{namespace}/{key}` | Update one system setting |
| `GET` | `/settings/v1/backend-api/revisions` | Query configuration revision history |

### Data Model

| Table | Responsibility |
| --- | --- |
| `stg_user_preference` | User-level preferences scoped by tenant, user, namespace, and key |
| `stg_tenant_config` | Tenant-level configuration scoped by tenant, namespace, and key |
| `stg_system_setting` | System-level defaults scoped by namespace, key, and optional region/scope |
| `stg_config_revision` | Full configuration change audit history |

Configuration resolution order is:

```text
system setting -> tenant config -> user preference
```

User preferences may override tenant defaults. Tenant configuration may override system defaults. System settings remain operator-controlled.

## 6. Security, Privacy, And Observability

- All protected routes use `sdkwork-iam-web-adapter` to inject `WebRequestContext`.
- Tenant and user scope come from the authenticated request context, not from generated SDK `tenant_id` or `user_id` request fields.
- User preferences require self access.
- Tenant configuration requires tenant administrator permissions.
- System settings require system administrator permissions.
- Sensitive configuration values use SDKWork-approved crypto utilities and must not be logged in plaintext.
- Configuration writes create `stg_config_revision` audit records.
- Process probes are mounted once by the listener: `/healthz`, `/livez`, `/readyz`, and `/metrics`.
- Logs, metrics, and traces carry framework-provided trace ids.

## 7. Deployment And Runtime Topology

SDKWork Settings uses the two-segment topology profile id format:

```text
<deploymentProfile>.<environment>
```

Active profiles:

| Profile id | Application public ingress | Platform plane |
| --- | --- | --- |
| `standalone.development` | `sdkwork-api-settings-standalone-gateway` | Optional embedded or local platform adapter |
| `standalone.production` | `sdkwork-api-settings-standalone-gateway` | Optional embedded or configured platform adapter |
| `cloud.development` | `sdkwork-api-settings-standalone-gateway` for the current application ingress | Optional `sdkwork-api-cloud-gateway` config bundle |
| `cloud.production` | `sdkwork-api-settings-standalone-gateway` for the current application ingress | `sdkwork-api-cloud-gateway` on `platform.api-gateway` |

Rules:

- `standalone` and `cloud` are the only deployment profiles.
- There is no split, service layout, or process-layout profile segment.
- Clients receive one `application.public-ingress` URL for Settings APIs.
- Platform APIs use `platform.api-gateway` when that plane is in scope.
- Internal process decomposition is an implementation detail and must not become a public script, profile id, SDK bootstrap, or route path axis.

## 8. Architecture Decision Index

- [ADR-0001: Settings Application Root Architecture](../decisions/ADR-0001-settings-application-root.md)

## 9. Verification

Relevant verification commands:

```bash
pnpm check:pnpm-script-standard
pnpm test:single-http-ingress
pnpm api:assembly:validate
pnpm gateway:route-composition:audit
pnpm test:api-response-envelope
pnpm db:validate
cargo test --workspace
```
