import { TauriPluginPinia } from '@tauri-store/pinia'
import { createPinia } from 'pinia'

const pinia = createPinia()

// 只保存到缓存目录，不进数据目录
pinia.use(
  TauriPluginPinia({
    autoStart: true,
    saveOnExit: true,
    saveOnChange: true,
  }),
)

export default pinia
export { useStore } from './default'
