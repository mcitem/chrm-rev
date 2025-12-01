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
import { RegisterInterceptor } from './lib/service'

const { state } = useColorMode()

console.log(packageJson.version)

const AppInner = defineComponent({
  setup() {
    onMounted(() => {
      const { message } = App.useApp()
      new RegisterInterceptor(message.error)
    })
    return () => <router-view />
  },
})
</script>
