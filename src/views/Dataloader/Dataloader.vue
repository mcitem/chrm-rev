<template>
  <div class="flex h-screen w-screen flex-col">
    <div class="flex-shrink-0 px-8 pb-6 pt-8">
      <a-steps :current="step" status="process" :items="items" />
    </div>
    <div class="min-h-0 flex-1 px-6">
      <InitDb v-if="step === 0" />
      <CheckDb v-else-if="step === 1" />
      <ImportItem v-else-if="step === 2" />
      <ImportStu v-else-if="step === 3" />
    </div>
  </div>

  <div v-if="step === 0 || step === 1" class="absolute bottom-9 right-9">
    <HelpButton type="primary" class="mr-2" />
    <a-button
      type="primary"
      :disabled="nextStepMutation.isPending.value"
      :loading="nextStepMutation.isPending.value"
      @click="() => nextStepMutation.mutate()"
    >
      {{ status === 'error' ? '重试' : '下一步' }}
    </a-button>
  </div>

  <div v-if="nextStepMutation.isPending.value" class="loading" />
  <!-- <a-table :data-source="dataSource" :columns="columns" /> -->
</template>

<script setup lang="ts">
import HelpButton from '@/components/HelpButton.vue'
import { DB_DIRTY, instance } from '@/lib/service'
import CheckDb from './CheckDb.vue'
import ImportItem from './ImportItem.vue'
import ImportStu from './ImportStu.vue'
import InitDb from './InitDb.vue'

const status = ref<'wait' | 'process' | 'finish' | 'error'>('process')
const error_text = ref('初始化失败，请重试')
const error_code = ref('')

const items = [
  {
    title: '初始化数据库',
    content: '初始化数据库文件',
  },
  {
    title: '检查数据库',
    content: '检查是否符合导入要求',
  },
  {
    title: '导入商品',
    content: '导入商品数据到数据库',
  },
  {
    title: '导入学生',
    content: '导入学生数据到数据库',
  },
] satisfies {
  title: string
  content?: string
}[]

const step = ref(0)

const nextStepMutation = useMutation({
  mutationFn: async () => {
    if (step.value === 0) {
      await instance.put('/sys/dataloader/init')
    }
    else if (step.value === 1) {
      await instance.post('/sys/dataloader/check')
    }
  },
  onError: (e) => {
    status.value = 'error'

    if (e.response?.data.msg) {
      error_text.value = e.response.data.msg
    }

    if (e.response?.data.error === DB_DIRTY) {
      error_code.value = DB_DIRTY
    }
  },
  onSuccess: () => {
    if (step.value === 0) {
      step.value = 1
      status.value = 'process'
      nextStepMutation.mutate()
    }
    else if (step.value === 1) {
      step.value = 2
      status.value = 'process'
    }
  },
})

onMounted(async () => {
  nextStepMutation.mutate()
})

export interface DataloaderContext {
  step: typeof step
  status: typeof status
  nextStepMutation: typeof nextStepMutation
  error_code: typeof error_code
  error_text: typeof error_text
}

provide<DataloaderContext>('dataloader', {
  step,
  status,
  nextStepMutation,
  error_code,
  error_text,
})
</script>
