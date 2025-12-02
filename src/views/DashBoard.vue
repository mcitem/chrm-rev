<!-- DashBoard组件 -->
<template>
  <div class="flex h-screen w-screen bg-[#ededed] dark:bg-[#1a1a1a]">
    <!-- 侧边栏 -->
    <div class="flex w-56 flex-shrink-0 flex-grow-0 flex-col">
      <div class="flex-grow-0 select-none py-5 text-center">
        <span class="font-mc10 text-4xl"> Chrm Rev </span>
      </div>
      <div class="flex flex-grow-0 select-none flex-col gap-3 px-4">
        <button
          v-for="(item, index) in DashBoardRoutes"
          :key="index"
          @click="
            () => {
              handleNavigation(item.name);
            }
          "
          :class="[
            'flex items-center justify-evenly rounded-md px-1 py-4 transition-all hover:bg-gray-100 hover:shadow-md dark:hover:bg-gray-700',
            route.name === item.name
              ? 'bg-green-500/20 hover:!bg-green-500/40 dark:bg-blue-800 dark:hover:!bg-blue-700'
              : '',
          ]"
        >
          <v-icon :icon="item.icon" />
          <div class="text-xl font-semibold text-gray-800 dark:text-gray-200">
            {{ item.title }}
          </div>
        </button>
      </div>
      <!-- 左侧侧边栏底部空白区 -->
      <div class="flex flex-grow flex-col items-center justify-end pb-2">
        <div><img src="/github.svg" width="80" /></div>
        <div class="flex items-center justify-center gap-2">
          <v-icon icon="mdi-github" size="small" class="inline-block" />
          <span
            class="inline-block transition-all hover:text-green-500/60 dark:hover:text-blue-200"
            @click="
              () => {
                invoke('open_web_url', {
                  url: 'https://github.com/mcitem/chrm-rev',
                });
              }
            "
          >
            mcitem/chrm-rev
          </span>
        </div>
        <div>
          {{ SysTime?.time }}
        </div>
      </div>
    </div>

    <!-- 内容区 -->
    <div class="flex-grow overflow-y-auto">
      <router-view v-slot="{ Component }">
        <keep-alive>
          <component :is="Component" />
        </keep-alive>
      </router-view>
    </div>
  </div>

  <Sudo />
  <TimeCheck />
</template>
<script setup lang="ts">
import Sudo from '@/components/Sudo.vue';
import TimeCheck from '@/components/TimeCheck.vue';
import { DashBoardRoutes } from '@/lib/router';
import { useSysTime } from '@/lib/service';
import { invoke } from '@tauri-apps/api/core';

const router = useRouter();
const route = useRoute();

const { data: SysTime } = useSysTime();

/**
 * 防抖和重复点击保护
 */
let isNavigating = false;
const handleNavigation = async (routeName: string) => {
  if (isNavigating || route.name === routeName) {
    return;
  }

  try {
    isNavigating = true;
    await router.replace({ name: routeName });
  } catch (error) {
    console.warn('Navigation cancelled or failed:', error);
  } finally {
    // 延迟重置标志，防止快速点击
    setTimeout(() => {
      isNavigating = false;
    }, 100);
  }
};
</script>
