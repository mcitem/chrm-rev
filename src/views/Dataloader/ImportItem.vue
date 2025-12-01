<template>
  <div class="flex size-full items-stretch justify-center pt-5">
    <ImportSiderBar />
    <a-divider orientation="vertical" class="h-full" />
    <SelectWorkSheet v-if="importStep === 0" />
    <SelectColumns v-else-if="importStep === 1" :options="OPTIONS" />
    <div v-else class="flex-1 pl-3">
      <div class="pb-1 text-xl font-bold">
        预览导入
      </div>
      <div class="pb-2 font-semibold">
        以下为第一行数据，请确认数据是否正确
      </div>
      <div class="pb-2 text-xs">
        若只有表头，请返回上一步勾选"数据包含表头"
      </div>
      <a-table
        :pagination="false"
        :columns="
          OPTIONS.map(i => {
            const replace = i.value.replace('i_', '')
            console.log(replace)
            return {
              title: i.label,
              key: replace,
              dataIndex: replace,
            }
          })
        "
        :data-source="
          checkImport.data.value?.first_item
            ? [checkImport.data.value?.first_item]
            : []
        "
      />

      <div class="pt-3 font-bold">
        预计导入
        <span> {{ checkImport.data.value?.total || 0 }}</span>
        条数据，确认无误后请点击完成导入
      </div>
    </div>
  </div>

  <div class="absolute bottom-9 right-9">
    <help-button class="mr-2" />
    <a-button
      class="mr-2"
      @click="
        () => {
          step = 3
        }
      "
    >
      跳过导入
    </a-button>
    <a-button
      class="mr-2"
      :disabled="importStep === 0"
      @click="
        () => {
          if (importStep === 1) {
            importStep = 0
            selectedSheet = ''
          }
          else if (importStep === 2) {
            importStep = 1
          }
        }
      "
    >
      上一步
    </a-button>

    <a-button
      type="primary"
      :disabled="nextStepDisabled"
      :loading="importStep === 2 && importItem.isPending.value"
      @click="
        () => {
          if (importStep === 0) {
            importStep = 1
          }
          else if (importStep === 1) {
            importStep = 2
            checkImport.mutate()
          }
          else if (importStep === 2) {
            // 导入
            importItem.mutate()
          }
        }
      "
    >
      下一步
    </a-button>
  </div>
</template>

<script setup lang="ts">
import type { DataloaderContext } from './Dataloader.vue'
import type {
  CheckItemImport,
  CheckItemImportReturn,
  LoadItemContext,
} from '@/bindings/dataloader'
import type { ImportContext } from '@/lib/context'
import { message } from 'antdv-next'
import { IMPORT_CONTEXT_KEY } from '@/lib/context'
import { instance } from '@/lib/service'

const { step } = inject<DataloaderContext>('dataloader')!

const OPTIONS = [
  {
    label: 'id',
    value: 'id',
    alias: ['id', 'ID'],
    required: false,
  },
  {
    label: '商品名称',
    value: 'i_name',
    alias: ['name', '商品名称', '名称'],
    required: true,
  },
  {
    label: '规格',
    value: 'i_spec',
    alias: ['spec', '规格'],
    required: true,
  },
  {
    label: '原价',
    value: 'i_p',
    alias: ['price', '原价'],
    required: true,
  },
  {
    label: '3折价（特别困难）',
    value: 'i_p_hard',
    alias: ['p_hard', '3折价', '特别困难'],
    required: true,
  },
  {
    label: '5折价（困难）',
    value: 'i_p_normal',
    alias: ['p_normal', '5折价', '困难'],
    required: true,
  },
  {
    label: '7折价（一般困难）',
    value: 'i_p_easy',
    alias: ['p_easy', '7折价', '一般困难'],
    required: true,
  },
  {
    label: '积分',
    value: 'i_p_score',
    alias: ['p_score', '积分'],
    required: true,
  },
]

const importStep = ref(0)
const path = ref('')
const selectedSheet = ref('')
const dataIncludeHeader = ref(true)
const loadContext = ref<Record<string, number | null>>({
  id: null,
  i_name: null,
  i_spec: null,
  i_p: null,
  i_p_hard: null,
  i_p_normal: null,
  i_p_easy: null,
  i_p_score: null,
} satisfies Record<keyof LoadItemContext, number | null>)

const checkImport = useMutation({
  mutationFn: async () => {
    const res = (await instance.post('/sys/dataloader/load/item/check', {
      path: path.value,
      sheet: selectedSheet.value,
      data_include_header: dataIncludeHeader.value,
      ctx: loadContext.value as LoadItemContext,
    } satisfies CheckItemImport)) as CheckItemImportReturn
    return res
  },
})

const nextStepDisabled = computed(() => {
  if (importStep.value === 0) {
    return !selectedSheet.value
  }
  else if (importStep.value === 1) {
    return OPTIONS.filter(option => option.required).some((option) => {
      return loadContext.value[option.value as keyof LoadItemContext] === null
    })
  }
  else if (importStep.value === 2) {
    return !checkImport.isSuccess.value
  }
  else {
    return false
  }
})

const importItem = useMutation({
  mutationFn: async () => {
    const res = (await instance.post('/sys/dataloader/load/item', {
      path: path.value,
      sheet: selectedSheet.value,
      data_include_header: dataIncludeHeader.value,
      ctx: loadContext.value as LoadItemContext,
    } satisfies CheckItemImport)) as number
    // console.log(res);
    return res
  },
  onSuccess: (res) => {
    message.success(`成功导入 ${res} 条数据`)
    step.value = 3
  },
})

provide<ImportContext>(IMPORT_CONTEXT_KEY, {
  path,
  selectedSheet,
  importStep,
  dataIncludeHeader,
  loadContext,
})
</script>
