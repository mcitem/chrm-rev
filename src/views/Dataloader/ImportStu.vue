<template>
  <div class="flex size-full items-stretch justify-center pt-5">
    <ImportSiderBar />
    <a-divider orientation="vertical" class="h-full" />
    <SelectWorkSheet v-if="importStep === 0" :auto="false" />
    <SelectColumns v-else-if="importStep === 1" :options="OPTIONS" />
    <div v-else class="flex-1 pl-3">
      <div class="pb-1 text-xl font-bold">
        预览导入
      </div>
      <a-alert
        v-if="checkImport.isError.value"
        class="mb-3"
        type="error"
        show-icon
        title="学生数据校验失败，无法导入"
        :description="
          (checkImport.error.value as AxiosError)?.response?.data?.msg
            || '请检查数据或列映射，返回上一步后重试。'
        "
      />
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
            const replace = i.value.replace('s_', '')
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

      <template v-if="checkImport.isSuccess.value">
        <div class="pb-2 pt-4 font-bold">
          请核对本次导入的学生人数
        </div>
        <a-table
          :pagination="false"
          :columns="countColumns"
          :data-source="[
            {
              key: 'counts',
              ...checkImport.data.value!.difficulty_counts,
              total: checkImport.data.value!.total,
            },
          ]"
        />
        <a-checkbox
          v-model:checked="countsConfirmed"
          class="mt-3"
          :disabled="importItem.isPending.value"
        >
          我已核对各困难等级人数及总人数，确认导入
        </a-checkbox>
      </template>
    </div>
  </div>

  <div class="absolute bottom-9 right-9">
    <help-button class="mr-2" />
    <a-button
      class="mr-2"
      :disabled="importItem.isPending.value"
      @click="
        () => {
          instance.post('/sys/dataloader/skip').then(() => {
            router.push({
              path: '/',
            })
          })
        }
      "
    >
      跳过导入
    </a-button>
    <a-button
      class="mr-2"
      :disabled="importItem.isPending.value"
      @click="
        () => {
          if (importStep === 0) {
            step = 2
          }
          else if (importStep === 1) {
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
            countsConfirmed = false
            checkImport.mutate()
          }
          else if (importStep === 2) {
            if (!nextStepDisabled) importItem.mutate()
          }
        }
      "
    >
      {{ importStep === 2 ? '完成导入' : '下一步' }}
    </a-button>
  </div>
</template>

<script setup lang="ts">
import type { DataloaderContext } from './Dataloader.vue'
import type {
  CheckStuImport,
  CheckStuImportReturn,
  LoadStuContext,
} from '@/bindings/dataloader'
import type { ImportContext } from '@/lib/context'
import type { AxiosError } from '@/lib/service'
import { message } from 'antdv-next'
import { IMPORT_CONTEXT_KEY } from '@/lib/context'
import { instance } from '@/lib/service'

const router = useRouter()
const queryClient = useQueryClient()

const { step } = inject<DataloaderContext>('dataloader')!

const OPTIONS = [
  {
    label: 'id',
    value: 'id',
    alias: ['id', 'ID'],
    required: false,
  },
  {
    label: '姓名',
    value: 's_name',
    alias: ['name', '姓名'],
    required: true,
  },
  {
    label: '学号',
    value: 's_no',
    alias: ['student_no', '学号'],
    required: true,
  },
  {
    label: '认定级别',
    value: 's_d_level',
    alias: ['difficulty_level', '认定级别'],
    required: true,
  },
  {
    label: '学院',
    value: 's_second_school',
    alias: ['secondary_school', '学院'],
    required: false,
  },
  {
    label: '班级',
    value: 's_class',
    alias: ['class', '班级'],
    required: false,
  },
  {
    label: '性别',
    value: 's_sex',
    alias: ['sex', '性别'],
    required: false,
  },
  {
    label: '专业',
    value: 's_major',
    alias: ['major', '专业'],
    required: false,
  },
  {
    label: '余额',
    value: 's_credit',
    alias: ['balance', '余额'],
    required: false,
  },
]

const importStep = ref(0)
const countsConfirmed = ref(false)
const countColumns = [
  { title: '不困难（人）', dataIndex: 'peaceful', key: 'peaceful' },
  { title: '一般困难（人）', dataIndex: 'easy', key: 'easy' },
  { title: '困难（人）', dataIndex: 'normal', key: 'normal' },
  { title: '特别困难（人）', dataIndex: 'hard', key: 'hard' },
  { title: '总人数（人）', dataIndex: 'total', key: 'total' },
]
const path = ref('')
const selectedSheet = ref('')
const dataIncludeHeader = ref(true)
const loadContext = ref<Record<string, number | null>>({
  id: null,
  s_name: null,
  s_no: null,
  s_d_level: null,
  s_second_school: null,
  s_class: null,
  s_sex: null,
  s_credit: null,
  s_major: null,
} satisfies Record<keyof LoadStuContext, number | null>)

const checkImport = useMutation({
  mutationFn: async () => {
    const res = (await instance.post('/sys/dataloader/load/stu/check', {
      path: path.value,
      sheet: selectedSheet.value,
      data_include_header: dataIncludeHeader.value,
      ctx: loadContext.value as unknown as LoadStuContext,
    } satisfies CheckStuImport)) as CheckStuImportReturn
    return res
  },
})

const importItem = useMutation({
  mutationFn: async () => {
    const res = (await instance.post(
      '/sys/dataloader/load/stu',
      {
        path: path.value,
        sheet: selectedSheet.value,
        data_include_header: dataIncludeHeader.value,
        ctx: loadContext.value as unknown as LoadStuContext,
      } satisfies CheckStuImport,
      {
        // Keep the existing loading state until the entire import commits.
        timeout: 0,
      },
    )) as number
    return res
  },
  onSuccess: async (res) => {
    await instance.post('/sys/dataloader/finish')
    await message.success(`成功导入 ${res} 条数据`)
    await queryClient.invalidateQueries()
    await router.push({
      path: '/',
    })
  },
})

const nextStepDisabled = computed(() => {
  if (importItem.isPending.value)
    return true
  if (importStep.value === 0) {
    return !selectedSheet.value
  }
  else if (importStep.value === 1) {
    return !OPTIONS.filter(option => option.required).every((option) => {
      const column = loadContext.value[option.value]
      return (
        typeof column === 'number' && Number.isInteger(column) && column >= 0
      )
    })
  }
  else if (importStep.value === 2) {
    return !checkImport.isSuccess.value || !countsConfirmed.value
  }
  else {
    return false
  }
})

provide<ImportContext>(IMPORT_CONTEXT_KEY, {
  path,
  selectedSheet,
  importStep,
  dataIncludeHeader,
  loadContext,
})
</script>
