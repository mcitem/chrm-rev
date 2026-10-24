<template>
  <a-config-provider
    :locale="zhCN"
    :theme="{
      algorithm:
        state === 'dark' ? theme.darkAlgorithm : theme.defaultAlgorithm,
    }"
  >
    <a-app>
      <AppInner />
    </a-app>
  </a-config-provider>
</template>

<script setup lang="tsx">
import packageJson from '@root/package.json'
import { useColorMode } from '@vueuse/core'
import { App, theme } from 'antdv-next'
import zhCN from 'antdv-next/locale/zh_CN'
import { databaseChanging } from './lib/databaseSession'
import { RegisterInterceptor } from './lib/service'

const { state } = useColorMode()

console.log(packageJson.version)

const AppInner = defineComponent({
  setup() {
    onMounted(() => {
      const { message } = App.useApp()
      new RegisterInterceptor(message.error)
    })
    return () => (
      <>
        <div class="h-full" inert={databaseChanging.value}>
          <router-view />
        </div>
        {databaseChanging.value && (
          <div
            class="fixed inset-0 z-[10000] flex items-center justify-center bg-black/30"
            role="status"
          >
            <div class="rounded-lg bg-white px-6 py-4 text-black shadow-lg">
              正在恢复备份，请稍候…
            </div>
          </div>
        )}
      </>
    )
  },
})
</script>
