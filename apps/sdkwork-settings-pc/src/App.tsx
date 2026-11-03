import { useEffect, useState, useSyncExternalStore } from 'react';
import {
  createDefaultSdkworkSettingsCatalog,
  createSdkworkBrowserSettingsStore,
  createSdkworkSettingsMessageCatalog,
  createSdkworkSettingsRuntime,
  SdkworkMemorySettingsStore,
  type SdkworkSettingDefinition,
  type SdkworkSettingsRuntime,
  type SdkworkSettingsSlotContext,
} from '@sdkwork/settings-core';
import {
  SdkworkSettingsCenter,
  SdkworkSettingsProvider,
} from '@sdkwork/settings-pc-react';
import '@sdkwork/settings-pc-react/styles.css';

const settingsCatalog = createDefaultSdkworkSettingsCatalog();
const settingsMessages = createSdkworkSettingsMessageCatalog({ locale: 'zh-CN' });

function createSettingsRuntime(): SdkworkSettingsRuntime {
  return createSdkworkSettingsRuntime({
    autoSaveDelayMs: 600,
    catalog: settingsCatalog,
    initialValues: {
      'about.version': '0.1.0',
    },
    query: {
      appId: 'sdkwork-settings',
      scope: 'user',
    },
    store: createSdkworkBrowserSettingsStore() ?? new SdkworkMemorySettingsStore(),
  });
}

/**
 * SDKWork Settings PC 应用。
 *
 * 设置中心本身由 `@sdkwork/settings-pc-react` 渲染，catalog、分层值、校验、
 * 保存模式与 slot 语义来自 `@sdkwork/settings-core`。本应用只负责运行时装配、
 * 主题同步和动作处理。
 */
export default function App() {
  const [runtime] = useState(createSettingsRuntime);
  const snapshot = useSyncExternalStore(
    runtime.subscribe,
    runtime.getSnapshot,
    runtime.getSnapshot,
  );
  const theme = snapshot.values['appearance.theme'];
  const explicitTheme = theme === 'light' || theme === 'dark' ? theme : undefined;

  useEffect(() => () => runtime.destroy(), [runtime]);

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle('dark', theme === 'dark');
    return () => {
      root.classList.remove('dark');
    };
  }, [theme]);

  function handleAction(
    settingId: string,
    _setting: SdkworkSettingDefinition,
    _context: SdkworkSettingsSlotContext,
  ): void {
    if (settingId === 'advanced.reset') {
      void runtime.reset();
    }
  }

  return (
    <div
      className="h-full min-h-0 bg-gray-50 p-4 dark:bg-neutral-900 sm:p-6"
      data-sdk-settings-theme={explicitTheme}
    >
      <SdkworkSettingsProvider
        catalog={settingsCatalog}
        locale="zh-CN"
        messages={settingsMessages}
        onAction={handleAction}
        runtime={runtime}
      >
        <SdkworkSettingsCenter
          description="管理账户、偏好与应用行为。"
          title="设置"
        />
      </SdkworkSettingsProvider>
    </div>
  );
}
