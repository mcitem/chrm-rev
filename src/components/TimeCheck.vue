<!-- 时间检查组件,应对bios没电时间小于2025 -->
<template>
  <transition
    enter-active-class="transition-opacity duration-300"
    leave-active-class="transition-opacity duration-300"
    enter-from-class="opacity-0"
    leave-to-class="opacity-0"
  >
    <div
      v-if="SysTime && SysTime.year < 2025 && !ignore"
      class="fixed inset-0 z-[999] flex items-center justify-center bg-[#00000077]"
    >
      <div class="rounded bg-[#eeeeee] px-4 py-6 shadow-lg dark:bg-[#2c2c2c]">
        <p class="mb-4">
          当前系统时间：<span class="font-mono">{{ SysTime?.time }}</span>
        </p>
        <p class="mb-4">请先修改到正确的系统时间再使用</p>
        <div>
          <v-btn
            @click="
              () => {
                ignore = true;
              }
            "
            color="red"
            >忽略</v-btn
          >
          <v-btn
            class="ml-2"
            color="primary"
            @click="
              () => {
                invoke('open_timedate_cpl');
              }
            "
            >修改系统时间</v-btn
          >
        </div>
      </div>
    </div>
    <div v-else></div>
  </transition>
</template>

<script setup lang="ts">
import { useSysTime } from '@/lib/service';
import { invoke } from '@tauri-apps/api/core';

const ignore = ref(false);

const { data: SysTime } = useSysTime();
</script>
