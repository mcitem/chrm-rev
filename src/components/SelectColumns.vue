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
        <template #headerCell="{ column }">
          <a-select
            v-model:value="
              // @ts-ignore
              column.match
            "
            :options="options"
            placeholder="请选择"
            allow-clear
            @change="
              (v: string | undefined) => {
                // @ts-ignore
                const col = column.col as number
                console.log(v, column)
                if (v !== undefined) {
                  const vv = v
                  loadContext[vv] = col

                  // 移除其他重复
                  table_header = [
                    ...table_header.map(h => {
                      if (h.col === col) {
                        h.match = v
                      }
                      else if (h.match === v) {
                        h.match = undefined
                      }
                      return h
                    }),
                  ]
                }
                else {
                  table_header = [
                    ...table_header.map(h => {
                      if (h.col === col) {
                        h.match = undefined
                      }
                      return h
                    }),
                  ]

                  Object.keys(loadContext)
                    .filter(k => {
                      return loadContext[k] === col
                    })
                    .forEach(k => {
                      loadContext[k] = null
                    })
                }
              }
            "
          />
        </template>
      </a-table>
    </div>
    <div class="text-gray-600 dark:text-gray-300">
      注：这里只展示前五行数据
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

const table_header = ref<
  {
    title: string
    dataIndex: string
    key: string
    col: number
    match: string | undefined
  }[]
>([])

const queryWorkSheetData = useQuery({
  retry: false,
  queryKey: ['/sys/dataloader/load/worksheet/data', path, selectedSheet],
  enabled: computed(() => !!path.value && !!selectedSheet.value),
  placeholderData: [],
  queryFn: async () => {
    const res = (await instance.post(`/sys/dataloader/load/worksheet/data`, {
      path: path.value,
      sheet: selectedSheet.value,
    })) as string[][]

    // 不消耗第一行数据
    const row1 = res[0] || []

    // 所有的index从0开始
    if (row1.length !== 0) {
      const row1_map = row1.map((i) => {
        const find = props.options.find(
          o => o.label === i || o.alias.includes(i),
        )
        if (find) {
          const col = row1.indexOf(i)
          loadContext.value[find.value] = col
          return { value: i, match: find.value }
        }

        return { value: i, match: undefined }
      })

      table_header.value = row1_map.map((cell, cellIndex) => ({
        title: cell.value,
        dataIndex: `col${cellIndex}`,
        key: `col${cellIndex}`,
        col: cellIndex,
        match: cell.match,
      }))
    }

    return res.map(row =>
      Object.fromEntries(
        row.map((cell, cellIndex) => [`col${cellIndex}`, cell]),
      ),
    )
  },
})
</script>
