<template>
  <div
    class="flex h-full items-center justify-center text-xl font-bold tracking-[0.2em]"
  >
    {{ status === 'error' ? '初始化失败，请重试' : '正在进行中，请勿关闭窗口' }}
  </div>
</template>

<script setup lang="ts">
import type { DataloaderContext } from './Dataloader.vue'

const { status } = inject<DataloaderContext>('dataloader')!
</script>
