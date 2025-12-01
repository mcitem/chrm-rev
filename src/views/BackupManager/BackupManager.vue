<template>
  <div
    class="flex h-screen w-screen items-center justify-center bg-gradient-to-br from-stone-50 via-white to-amber-50 px-6 dark:from-stone-950 dark:via-stone-900 dark:to-stone-950"
  >
    <div
      class="bg-white/90 w-full max-w-xl rounded-2xl border border-stone-200 p-8 shadow-[0_20px_60px_rgba(120,113,108,0.12)] backdrop-blur dark:border-stone-700 dark:bg-stone-900/90 dark:shadow-[0_20px_60px_rgba(0,0,0,0.35)]"
    >
      <div class="text-2xl font-semibold text-stone-900 dark:text-stone-50">
        没有检测到配置和数据
      </div>
      <div class="mt-3 text-base leading-7 text-stone-600 dark:text-stone-300">
        这似乎是第一次启动。若你需要从其他设备迁移，或者手头已有备份文件，可以直接恢复备份；如果没有备份文件，请继续初始化流程。
      </div>

      <div class="mt-8 flex flex-wrap gap-3">
        <a-button
          type="primary"
          :loading="restoring"
          @click="handleRestoreBackup"
        >
          通过备份文件恢复
        </a-button>
        <a-button :disabled="restoring" @click="continueInit">
          继续初始化流程
        </a-button>
      </div>
    </div>
  </div>

  <div class="z-9998 absolute bottom-8 right-6 px-4 py-2">
    <a-button
      type="default"
      class="bg-white/80 mr-2 border-stone-300 text-stone-700 shadow-sm dark:border-stone-600 dark:bg-stone-800/80 dark:text-stone-100"
      @click="() => instance.post('/sys/open_docs', {})"
    >
      <template #icon>
        <QuestionOutlined />
      </template>
    </a-button>
  </div>
</template>

<script setup lang="ts">
import type { RollBackBackup } from '@/bindings/sys'
import { QuestionOutlined } from '@antdv-next/icons'
import { open } from '@tauri-apps/plugin-dialog'
import { App } from 'antdv-next'
import { instance } from '@/lib/service'

const queryClient = useQueryClient()
const { message } = App.useApp()
const router = useRouter()

const restoring = ref(false)

async function handleRestoreBackup() {
  if (restoring.value)
    return

  restoring.value = true

  try {
    const path = await open({
      multiple: false,
      title: '选择备份文件',
      directory: false,
      filters: [
        {
          name: 'Zip Files',
          extensions: ['zip'],
        },
      ],
    })

    if (!path)
      return

    await instance.post('/sys/backup/rollback', {
      path: path as string,
    } satisfies RollBackBackup)

    message.success('恢复成功')
    await queryClient.invalidateQueries()
    router.push({ path: '/dashboard' })
  }
  catch (e) {
    message.error(`恢复失败：${e}`)
  }
  finally {
    restoring.value = false
  }
}
async function continueInit() {
  await queryClient.invalidateQueries()
  router.push({ path: '/configloader' })
}
</script>
