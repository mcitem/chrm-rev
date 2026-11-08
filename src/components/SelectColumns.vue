<template>
  <div class="flex min-h-0 min-w-0 flex-1 flex-col self-stretch py-5 pl-2">
    <a-checkbox v-model:checked="dataIncludeHeader" class="mb-3">
      数据包含表头
    </a-checkbox>
    <div class="min-h-0 min-w-0 flex-1 overflow-auto">
      <a-table
        :scroll="{ x: 'max-content' }"
        class="w-full"
        :columns="table_header"
        :data-source="queryWorkSheetData.data.value"
        :pagination="false"
      >
        <template #headerCell="{ index }">
          <a-select
            :value="table_header[index]?.match ?? '__ignore__'"
            :options="columnOptions"
            placeholder="不导入此列"
            allow-clear
            @change="(value: string | undefined) => updateMatch(index, value)"
          />
        </template>
      </a-table>
    </div>
    <div class="text-gray-600 dark:text-gray-300">
      注：这里只展示前五行数据；只需配对必填字段，其他列可以不选
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ImportContext } from '@/lib/context'
import { IMPORT_CONTEXT_KEY } from '@/lib/context'

const props = defineProps<{
  options: {
    label: string
    value: string
    alias: string[]
    required?: boolean
  }[]
}>()

const { path, selectedSheet, dataIncludeHeader, loadContext }
  = inject<ImportContext>(IMPORT_CONTEXT_KEY)!

const columnOptions = computed(() => [
  { label: '不导入此列', value: '__ignore__' },
  ...props.options,
])

const table_header = ref<
  {
    title: string
    dataIndex: string
    key: string
    col: number
    match: string | undefined
  }[]
>([])

function rebuildContext() {
  const context = Object.fromEntries(
    props.options.map(option => [option.value, null]),
  ) as Record<string, number | null>
  for (const header of table_header.value) {
    if (header.match && header.match in context)
      context[header.match] = header.col
  }
  loadContext.value = context
}

function updateMatch(column: number, value: string | undefined) {
  const match = props.options.some(option => option.value === value)
    ? value
    : undefined
  table_header.value = table_header.value.map(header => ({
    ...header,
    match:
      header.col === column
        ? match
        : header.match === match
          ? undefined
          : header.match,
  }))
  rebuildContext()
}

const queryWorkSheetData = useQuery({
  retry: false,
  staleTime: 0,
  refetchOnMount: 'always',
  refetchOnWindowFocus: false,
  refetchOnReconnect: false,
  queryKey: ['/sys/dataloader/load/worksheet/data', path, selectedSheet],
  enabled: computed(() => !!path.value && !!selectedSheet.value),
  placeholderData: [],
  queryFn: async () => {
    table_header.value = []
    rebuildContext()
    const requestedPath = path.value
    const requestedSheet = selectedSheet.value
    const res = (await instance.post(`/sys/dataloader/load/worksheet/data`, {
      path: path.value,
      sheet: selectedSheet.value,
    })) as string[][]

    if (path.value !== requestedPath || selectedSheet.value !== requestedSheet)
      return []
    const row1 = res[0] || []
    table_header.value = row1.map((cell, index) => ({
      title: cell,
      dataIndex: `col${index}`,
      key: `col${index}`,
      col: index,
      match: props.options.find(o => o.label === cell || o.alias.includes(cell))
        ?.value,
    }))

    const seen = new Set<string>()
    for (const header of table_header.value) {
      if (header.match && seen.has(header.match))
        header.match = undefined
      if (header.match)
        seen.add(header.match)
    }
    rebuildContext()

    return res.map(row =>
      Object.fromEntries(
        row.map((cell, cellIndex) => [`col${cellIndex}`, cell]),
      ),
    )
  },
})
</script>
