import { TauriPluginPinia } from '@tauri-store/pinia';
import { createPinia } from 'pinia';

const pinia = createPinia();

pinia.use(
  TauriPluginPinia({
    autoStart: true,
  })
);

export default pinia;
export { useStore } from './default';
