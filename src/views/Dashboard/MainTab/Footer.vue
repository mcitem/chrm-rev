<template>
  <div
    class="flex-shrink-0 px-3 py-4 text-right"
    style="background: var(--ant-table-footer-bg)"
  >
    共 {{ records?.summary.stu_count }} 人，总计（折后价）
    {{ records?.summary.total_discount_price }} 元
  </div>
</template>

<script setup lang="ts">
import { useAllRecordsWithSummary } from '@/lib/service'

const { data: records } = useAllRecordsWithSummary()
</script>
