<template>
  <div
    :class="
      cn(
        `flex h-full flex-col items-center justify-center gap-3 font-bold`,
        status !== 'error' && 'tracking-[0.2em]',
        status === 'error' && 'text-xl',
      )
    "
  >
    <div>
      {{ status === 'error' ? error_text : '正在进行中，请勿关闭窗口' }}
    </div>
    <div v-if="status === 'error' && next_pk_" class="text-xs text-gray-200">
      当前数据库已有 {{ next_pk_?.item }} 条商品数据，
      {{ next_pk_?.stu }} 条学生数据
    </div>
    <div>
      <a-button
        danger
        class="mr-2 text-red"
        @click="
          () => {
            step = 2
          }
        "
      >
        忽略风险，继续导入
      </a-button>
      <a-button
        :disabled="backupAndClearMutation.isPending.value"
        :loading="backupAndClearMutation.isPending.value"
        @click="() => backupAndClearMutation.mutate()"
      >
        备份并清除数据库
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { DataloaderContext } from './Dataloader.vue'
import type { GetNextPk } from '@/bindings/dataloader'
import type { UnsafeInvokeVertify } from '@/bindings/sys'
import { App } from 'antdv-next'
import { cn } from '@/lib'

const { message } = App.useApp()

const { status, step, nextStepMutation, error_code, error_text }
  = inject<DataloaderContext>('dataloader')!

const next_pk_ = ref<GetNextPk>()
onMounted(async () => {
  const res = (await instance.get('/sys/dataloader/next_pk')) as GetNextPk
  next_pk_.value = res

  if (res.item >= 2 && res.stu === 1) {
    step.value = 3
  }
})

const backupAndClearMutation = useMutation({
  mutationFn: async () => {
    const res = await instance.post<string, string>('/sys/backup/create')
    await instance.post('/sys/unsafe/clear_db_file', {
      secret: window.UNSAFE_INVOKE_SECRET,
    } satisfies UnsafeInvokeVertify)
    return res
  },
  onSuccess: (path) => {
    message.success(`备份成功，已保存到桌面，文件：${path}`)
    if (step.value === 1) {
      step.value = 0
      error_code.value = ''
      error_text.value = '初始化失败，请重试'
      status.value = 'process'
      nextTick(() => {
        nextStepMutation.mutate()
      })
    }
  },
})
</script>
