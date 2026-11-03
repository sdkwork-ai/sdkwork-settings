# SDKWork Settings PC

Browser settings center for SDKWork. The surface is composed from the shared
appbase components, not an app-local settings implementation.

## Composition

```text
App.tsx
  -> @sdkwork/settings-pc-react
       SdkworkSettingsProvider
       SdkworkSettingsCenter
  -> @sdkwork/settings-core
       catalog / layered values / validation / save modes / slots
  -> SdkworkSettingsStorePort
       browser-local adapter today; generated Settings App SDK adapter later
```

`App.tsx` owns only runtime assembly, theme synchronization, and action handling.
It does not own navigation, row controls, validation, dirty tracking, or save
semantics.

## Development

```bash
pnpm install
pnpm --dir apps/sdkwork-settings-pc dev:standalone
pnpm --dir apps/sdkwork-settings-pc build
pnpm --dir apps/sdkwork-settings-pc lint
```

The browser application persists user preferences through the core browser
store. When the generated Settings App SDK is wired, implement
`SdkworkSettingsStorePort` in the app runtime layer and pass it to
`createSdkworkSettingsRuntime`; the UI stays unchanged.
