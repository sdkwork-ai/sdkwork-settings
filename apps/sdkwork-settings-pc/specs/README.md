# Module Specs

本目录是 `@sdkwork/sdkwork-settings-pc` 的本地 SDKWork 组件契约说明。

- 组件根:`sdkwork-settings/apps/sdkwork-settings-pc`
- 上级清单:`../../agent.app.config.json`
- 规范索引:`../../../sdkwork-specs/README.md`

## 适用规范

PC 应用骨架遵循以下 SDKWork 规范(按需加载,不要整库拷贝):

- `sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md` — PC 应用根架构与目录布局
- `sdkwork-specs/FRONTEND_CODE_SPEC.md` — 前端代码(UI → 服务 → SDK 流向)
- `sdkwork-specs/TYPESCRIPT_CODE_SPEC.md` — TypeScript 代码规范(strict 模式)
- `sdkwork-specs/TAILWIND_CSS_INTEGRATION_SPEC.md` — Tailwind CSS v4 集成规范
- `sdkwork-specs/NAMING_SPEC.md` / `sdkwork-specs/CODE_STYLE_SPEC.md` — 命名与代码风格

## 当前阶段

本骨架处于初始阶段,仅包含:

- 顶部导航栏、左侧分区导航、右侧内容区的经典设置布局
- 用户偏好面板(外观 / 语言 / 通知 / 隐私),使用本地 React state
- 租户配置、系统设置分区的占位实现

后续阶段将按 `APP_PC_ARCHITECTURE_SPEC.md` 拆分 `packages/` 下的 `pc-core`、
`pc-commons`、`pc-shell` 与 `pc-<capability>` 包族,并接入生成的 App SDK 客户端
与 appbase IAM 运行时。在接入 SDK 前,UI 不直接发起原始 HTTP 请求或手动拼装认证头。

## 配置与运行时

- 浏览器公开运行时配置示例:`config/browser/runtime-env.development.example.json`
- 本地覆盖:复制为 `.env.local`(已被 `.gitignore` 忽略)
- 开发命令:`pnpm dev`(在仓库根目录执行,通过工作区解析依赖)

不要将根规范文本拷贝到本目录;请链接到 `../../../sdkwork-specs/` 下的文件。
