<template>
  <div class="h-full flex-1 pl-2 pt-5">
    <div class="pb-3 text-xl font-semibold">
      请选择要导入的Excel文件(.xlsx/.xls)
    </div>
    <a-button
      type="primary"
      @click="
        () => {
          open({
            multiple: false,
            filters: [{ name: 'Excel Files', extensions: ['xlsx', 'xls'] }],
          }).then(p => {
            if (p) {
              path = p
            }
            else {
              path = ''
              selectedSheet = ''
            }
          })
        }
      "
    >
      选择文件
    </a-button>
    <div>
      <div v-if="path" class="pt-3 text-sm text-gray-700 dark:text-gray-200">
        已选文件：{{ path }}
      </div>
    </div>
    <div
      v-if="
        path
          && queryWorkSheet.data.value
          && queryWorkSheet.data.value.length > 0
      "
      class="pt-3"
    >
      <a-select
        v-model:value="selectedSheet"
        :options="queryWorkSheet.data.value"
        placeholder="选择工作表"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ImportContext } from '@/lib/context'
import { listen, TauriEvent } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { IMPORT_CONTEXT_KEY } from '@/lib/context'

const props = withDefaults(
  defineProps<{
    auto?: boolean
  }>(),
  {
    auto: true,
  },
)

const { path, selectedSheet } = inject<ImportContext>(IMPORT_CONTEXT_KEY)!

let unlisten: () => void = () => {}
onMounted(async () => {
  unlisten = await listen<{ paths: string[] }>(TauriEvent.DRAG_DROP, (e) => {
    const p = e.payload.paths[0]
    if (p) {
      path.value = p
    }
    else {
      path.value = ''
      selectedSheet.value = ''
    }
  })
})

onUnmounted(() => {
  unlisten()
})

const queryWorkSheet = useQuery<{ value: string }[]>({
  retry: false,
  placeholderData: [],
  enabled: computed(() => path.value !== undefined && path.value !== ''),
  queryKey: ['/sys/dataloader/load/worksheet', path],
  queryFn: async () => {
    const res = (await instance.post('/sys/dataloader/load/worksheet', {
      path: path.value,
    })) as string[]

    if (Array.isArray(res)) {
      if (res.length === 1 && props.auto) {
        selectedSheet.value = res[0]
      }
      return res.map(v => ({ value: v }))
    }
    else {
      return []
    }
  },
})
</script>
