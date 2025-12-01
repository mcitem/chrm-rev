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
            checkImport.mutate()
          }
          else if (importStep === 2) {
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
  CheckStuImport,
  CheckStuImportReturn,
  LoadStuContext,
} from '@/bindings/dataloader'
import type { ImportContext } from '@/lib/context'
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

const nextStepDisabled = computed(() => {
  if (importStep.value === 0) {
    return !selectedSheet.value
  }
  else if (importStep.value === 1) {
    return OPTIONS.filter(option => option.required).some((option) => {
      return loadContext.value[option.value as keyof LoadStuContext] === null
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
    const res = (await instance.post('/sys/dataloader/load/stu', {
      path: path.value,
      sheet: selectedSheet.value,
      data_include_header: dataIncludeHeader.value,
      ctx: loadContext.value as unknown as LoadStuContext,
    } satisfies CheckStuImport)) as number
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

provide<ImportContext>(IMPORT_CONTEXT_KEY, {
  path,
  selectedSheet,
  importStep,
  dataIncludeHeader,
  loadContext,
})
</script>
