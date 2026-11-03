# Module Specs

本目录是 `@sdkwork/sdkwork-settings-pc` 的本地 SDKWork 组件契约说明。

- 组件根：`sdkwork-settings/apps/sdkwork-settings-pc`
- 上级清单：`../../agent.app.config.json`
- 规范索引：`../../../sdkwork-specs/README.md`
- 共享设置组件契约：`../../../sdkwork-appbase/specs/settings.spec.yaml`

## 适用规范

PC 应用骨架遵循以下 SDKWork 规范（按需加载，不要整库拷贝）：

- `sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md` — PC 应用根架构与目录布局
- `sdkwork-specs/APP_PC_REACT_UI_SPEC.md` — PC React 包拆分与 app SDK 边界
- `sdkwork-specs/FRONTEND_CODE_SPEC.md` — 前端代码（UI → 服务 → SDK 流向）
- `sdkwork-specs/TYPESCRIPT_CODE_SPEC.md` — TypeScript 代码规范（strict 模式）
- `sdkwork-specs/I18N_SPEC.md` — 语言与消息目录
- `sdkwork-specs/NAMING_SPEC.md` / `sdkwork-specs/CODE_STYLE_SPEC.md` — 命名与代码风格

## 当前实现

- 设置中心由 `@sdkwork/settings-pc-react` 的 `SdkworkSettingsProvider` +
  `SdkworkSettingsCenter` 渲染，不再维护应用私有的导航、行控件与保存状态。
- catalog、分层值、校验、save mode、dirty 状态、重启提示和 slot 语义来自
  `@sdkwork/settings-core`，应用只负责运行时装配、主题同步和动作处理。
- 当前使用 `@sdkwork/settings-core` 的浏览器存储适配器作为用户偏好的本地持久层；
  接入生成的 Settings App SDK 后，将实现 `SdkworkSettingsStorePort` 注入运行时即可，
  UI 组件无需改动。
- 全量设置项、分组、slot 名称与 12 种 `appType` 的适配矩阵见
  `sdkwork-appbase/specs/settings.spec.yaml`。

## 配置与运行时

- 浏览器公开运行时配置示例：`config/browser/runtime-env.development.example.json`
- 本地覆盖：复制为 `.env.local`（已被 `.gitignore` 忽略）
- 开发命令：`pnpm dev`（在仓库根目录执行，通过工作区解析依赖）

不要将根规范文本拷贝到本目录；请链接到 `../../../sdkwork-specs/` 下的文件。
