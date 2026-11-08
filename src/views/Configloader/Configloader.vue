<template>
  <a-splitter class="h-screen" :class="{ 'pointer-events-none': saving }">
    <a-splitter-panel default-size="30%" min="20%" max="80%">
      <VueMonacoEditor
        v-model:value="config_string"
        :theme="theme"
        default-language="json"
        :options="{ readOnly: saving }"
      />
    </a-splitter-panel>
    <a-splitter-panel :style="{ padding: '4px 12px' }">
      <div class="text-xl font-bold">
        编辑系统配置
      </div>
      <div class="pb-3 font-serif">
        编辑并保存配置到数据库以继续
      </div>
      <div v-if="validationError" class="text-red-600 pb-3">
        {{ validationError }}
      </div>
      <div class="pb-3">
        员工模板（示例：张三）
        <DynamicTags v-model="userTemplate" />
      </div>
      <div class="pb-3">
        时间模板（示例：09:30-11:00）
        <DynamicTags v-model="timeTemplate" />
      </div>

      <div class="py-2">
        <div class="pb-2">
          导出路径
        </div>
        <a-input v-model:value="path" placeholder="请选择或填写 CSV 文件的完整路径" />
        <a-button
          class="mt-2"
          :disabled="!config_v2 || validation !== 'valid' || saving || selectingExportPath"
          @click="exportPathModalOpen = true"
        >
          配置导出路径
        </a-button>
        <div class="pt-2 text-xs text-gray-600 dark:text-gray-300">
          电子表使用 CSV 格式，可用 Excel 打开；每次导出会追加到所选文件。
        </div>
      </div>

      <div class="flex gap-2 pb-3">
        使用兼容旧版导出格式
        <a-switch v-model:checked="legacyExportFormat" />
      </div>

      <div class="">
        <div class="pb-2 pt-3 text-lg font-semibold">
          业务配置（无特殊需求保持默认）
        </div>
        <div class="pb-2 font-bold">
          余额配置
        </div>

        <div class="flex gap-2 pb-3">
          是否按原价扣除余额（关闭后按折后价金额扣除余额）
          <a-switch v-model:checked="balancePayFor" />
        </div>

        <div class="font-semibold">
          基础额度设置
        </div>

        <div class="pb-3">
          不困难
          <a-input
            v-model:value="defaultPeaceful"
            placeholder="默认为 0.00元"
            style="width: 150px"
          />
        </div>
        <div class="pb-3">
          一般困难
          <a-input
            v-model:value="defaultEasy"
            placeholder="默认为 70.00元"
            style="width: 150px"
          />
        </div>
        <div class="pb-3">
          困难
          <a-input
            v-model:value="defaultNormal"
            placeholder="默认为 100.00元"
            style="width: 150px"
          />
        </div>
        <div class="pb-3">
          特别困难
          <a-input
            v-model:value="defaultHard"
            placeholder="默认为 150.00元"
            style="width: 150px"
          />
        </div>
      </div>
    </a-splitter-panel>
  </a-splitter>

  <ExportPathModal
    v-model:open="exportPathModalOpen"
    v-model:selecting="selectingExportPath"
    hint="完成后请点击“保存配置”，使导出路径生效。"
    @selected="applyExportPath"
  />

  <div class="z-9998 absolute bottom-8 right-6 px-4 py-2">
    <a-button
      type="default"
      class="mr-2"
      @click="() => instance.post('/sys/open_docs', {})"
    >
      <template #icon>
        <QuestionOutlined />
      </template>
    </a-button>
    <a-button
      type="primary"
      :disabled="!config_v2 || validation !== 'valid' || selectingExportPath"
      :loading="saving"
      @click="saveConfig"
    >
      保存配置
    </a-button>
  </div>
</template>

<script setup lang="ts">
import type { ConfigInner } from '@/bindings/config_v2'
import { QuestionOutlined } from '@antdv-next/icons'
import { VueMonacoEditor } from '@guolao/vue-monaco-editor'
import { listen, TauriEvent } from '@tauri-apps/api/event'
import { useColorMode } from '@vueuse/core'
import { App } from 'antdv-next'
import DynamicTags from '@/components/DynamicTags.vue'
import ExportPathModal from '@/components/ExportPathModal.vue'
import { instance } from '@/lib/service'
import '@/lib/editor'

const queryClient = useQueryClient()
const { message } = App.useApp()

const router = useRouter()

const config_v2 = ref<ConfigInner>()

const config_string = ref<string>('')

function sync() {
  config_string.value = JSON.stringify(config_v2.value, null, 2)
}

const validation = ref<'pending' | 'valid' | 'invalid'>('pending')
const validationError = ref('')
const saving = ref(false)
const exportPathModalOpen = ref(false)
const selectingExportPath = ref(false)
let revision = 0

const path = computed({
  get: () => config_v2.value?.export_path ?? '.csv',
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.export_path = value
    sync()
  },
})

function applyExportPath(selected: string) {
  path.value = selected
  exportPathModalOpen.value = false
  message.success('已设置导出路径，请点击“保存配置”使路径生效')
}
watch(
  config_string,
  async (text) => {
    const current = ++revision
    validation.value = 'pending'
    try {
      const parsed = (await instance.post(
        '/sys/validate/config',
        JSON.parse(text),
      )) as ConfigInner
      if (current !== revision)
        return
      config_v2.value = parsed
      validation.value = 'valid'
      validationError.value = ''
    }
    catch {
      if (current !== revision)
        return
      validation.value = 'invalid'
      validationError.value = '配置无效，请检查 JSON 格式和各字段数值后再保存'
    }
  },
  { flush: 'sync' },
)

async function saveConfig() {
  if (saving.value || selectingExportPath.value || validation.value !== 'valid')
    return
  saving.value = true
  try {
    const text = config_string.value
    const parsed = (await instance.post(
      '/sys/validate/config',
      JSON.parse(text),
    )) as ConfigInner
    if (text !== config_string.value)
      throw new Error('配置已变化，请重新保存')
    await instance.put('/sys/config', parsed)
    message.success('保存配置成功')
    await queryClient.invalidateQueries()
    await router.push({ path: '/' })
  }
  catch (error) {
    message.error(`保存配置失败：${error}`)
  }
  finally {
    saving.value = false
  }
}

const { state } = useColorMode()

const theme = computed(() => (state.value === 'dark' ? 'vs-dark' : 'vs'))

let unlisten: () => void = () => {}
onMounted(async () => {
  try {
    config_v2.value = await instance.get('/sys/config')
    sync()
  }
  catch (e) {
    message.error(`加载配置失败：${e}`)
  }
  unlisten = await listen<{ paths: string[] }>(TauriEvent.DRAG_DROP, (e) => {
    const p = e.payload.paths[0]
    if (p) {
      path.value = p
    }
    else {
      path.value = ''
    }
  })
})

onUnmounted(() => {
  unlisten()
})

const legacyExportFormat = computed({
  get: () => config_v2.value?.legacy_export_format ?? true,
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.legacy_export_format = value
    sync()
  },
})

const timeTemplate = computed<string[]>({
  get: () => config_v2.value?.time_template ?? [],
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.time_template = value
    sync()
  },
})

const userTemplate = computed<string[]>({
  get: () => config_v2.value?.user_template ?? [],
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.user_template = value
    sync()
  },
})

const balancePayFor = computed({
  get: () => config_v2.value?.balance_config.pay_for_original_price ?? true,
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.balance_config.pay_for_original_price = value
    sync()
  },
})

const defaultPeaceful = computed({
  get: () =>
    config_v2.value?.balance_config.default_balance.peaceful_balance ?? '0.00',
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.balance_config.default_balance.peaceful_balance = value
    sync()
  },
})

const defaultEasy = computed({
  get: () =>
    config_v2.value?.balance_config.default_balance.easy_balance ?? '70.00',
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.balance_config.default_balance.easy_balance = value
    sync()
  },
})

const defaultNormal = computed({
  get: () =>
    config_v2.value?.balance_config.default_balance.normal_balance ?? '100.00',
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.balance_config.default_balance.normal_balance = value
    sync()
  },
})

const defaultHard = computed({
  get: () =>
    config_v2.value?.balance_config.default_balance.hard_balance ?? '150.00',
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.balance_config.default_balance.hard_balance = value
    sync()
  },
})
</script>
