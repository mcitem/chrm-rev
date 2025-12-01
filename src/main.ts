import { VueQueryPlugin } from '@tanstack/vue-query'
import { createApp } from 'vue'
import App from '@/App.vue'
import router from '@/lib/router'
import pinia from '@/lib/stores'
import './assets/index.css'
import '@/lib/editor'

const app = createApp(App)

app.use(pinia)
app.use(router)
app.use(VueQueryPlugin, {
  enableDevtoolsV6Plugin: true,
})

app.mount('#root')

document.addEventListener('contextmenu', (event) => {
  event.preventDefault()
})
