<template>
  <RouterView />
  <!-- <VueQueryDevtools button-position="bottom-left" /> -->
  <Toaster :theme="state" expand />
</template>

<script setup lang="ts">
import { useStore } from '@/lib/stores';
import {
  DefaultError,
  QueryObserverResult,
  RefetchOptions,
} from '@tanstack/vue-query';
// import { VueQueryDevtools } from '@tanstack/vue-query-devtools';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useColorMode } from '@vueuse/core';
import { Toaster } from 'vue-sonner';
import { useTheme } from 'vuetify/lib/composables/theme.mjs';

export interface AppContext {
  refetch: (
    options?: RefetchOptions | undefined
  ) => Promise<QueryObserverResult<boolean, DefaultError>>;
}

// 用于等待vue加载完后再显示窗口，避免白屏
const main = getCurrentWebviewWindow();

// 验证迁移状态，并跳转到首页
const MigrateStatusQuery = useQuery<boolean>({
  queryKey: ['/migration/status'],
  queryFn: () => instance.get('/migration/status'),
});

provide<AppContext>('AppContext', {
  refetch: MigrateStatusQuery.refetch,
});

const { data: MigrateStatus } = MigrateStatusQuery;

const currentRoute = useRoute();
watch(MigrateStatus, newValue => {
  if (newValue === undefined) return;
  // V=true表示已经迁移完成，跳转到首页
  // V=false表示未迁移，跳转到初始化页面
  const targetPath = newValue ? '/dashboard/index' : '/init';
  if (currentRoute.path !== targetPath) {
    console.log('Navigating to', targetPath);
    router.replace(targetPath).then(() => {
      main.show();
    });
  } else {
    main.show();
  }
});

// F5刷新时保存TAB数据
const store = useStore();
window.addEventListener('beforeunload', () => {
  store.$tauri.saveAllNow();
});

// 主题同步到vuetify
const { state } = useColorMode();
const vuetifyTheme = useTheme();
watch(
  state,
  newVal => {
    vuetifyTheme.change(newVal);
  },
  { immediate: true }
);
</script>
