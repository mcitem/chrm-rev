<template>
  <div class="flex-shrink-0 self-start pr-2 pt-5">
    <a-steps
      class="text-xl font-bold"
      orientation="vertical"
      :current="importStep"
      :items="[
        {
          title: '选择工作表',
        },
        {
          title: '选择列',
        },
        {
          title: '确认导入',
        },
      ]"
    />

    <a-divider class="w-full" />
    <div
      v-if="selectedSheet"
      class="w-[120px] text-gray-700 dark:text-gray-200"
    >
      已选工作表：
      <a-popover>
        <template #content>
          {{ path }}#{{ selectedSheet }}
        </template>
        <div class="truncate font-semibold">
          {{ path }}#{{ selectedSheet }}
        </div>
      </a-popover>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ImportContext } from '@/lib/context'
import { IMPORT_CONTEXT_KEY } from '@/lib/context'

const { path, selectedSheet, importStep }
  = inject<ImportContext>(IMPORT_CONTEXT_KEY)!
</script>
