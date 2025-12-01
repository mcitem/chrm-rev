<template>
  <a-splitter class="h-screen">
    <a-splitter-panel default-size="45%" min="20%" max="80%">
      <VueMonacoEditor
        v-model:value="config_string"
        :theme="theme"
        default-language="json"
        @update:value="onConfigStringChange"
      />
    </a-splitter-panel>
    <a-splitter-panel :style="{ padding: '4px 12px' }">
      <div class="text-xl font-bold">
        编辑配置文件
      </div>
      <div class="pb-3 font-serif">
        编辑并保存配置文件以继续
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
        导出路径
        <a-input v-model:value="path" placeholder=".csv" style="width: 150px" />
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

  <div class="z-9998 absolute bottom-8 right-6 px-4 py-2">
    <a-button
      type="default"
      class="mr-2"
      @click="() => instance.post('/sys/open_docs')"
    >
      <template #icon>
        <QuestionOutlined />
      </template>
    </a-button>
    <a-button
      type="primary"
      @click="
        () =>
          instance
            .put('/sys/config', config_v2)
            .catch(e => {
              message.error(`保存配置失败：${e}`)
            })
            .then(async () => {
              message.success('保存配置成功')
              await queryClient.invalidateQueries();
              router.push({ path: '/' })
            })
      "
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
import { instance } from '@/lib/service'

const queryClient = useQueryClient()
const { message } = App.useApp()

const router = useRouter()

const config_v2 = ref<ConfigInner>()

const config_string = ref<string>('')

function sync() {
  config_string.value = JSON.stringify(config_v2.value, null, 2)
}

async function onConfigStringChange(newValue: string) {
  try {
    const object = JSON.parse(newValue)

    const parsed = (await instance.post(
      '/sys/validate/config',
      object,
    )) as ConfigInner

    config_v2.value = parsed
  }
  catch (e) {
    console.error(e)
  }
}

const { state } = useColorMode()

const theme = computed(() => (state.value === 'dark' ? 'vs-dark' : 'vs'))

const path = computed({
  get: () => config_v2.value?.export_path ?? '.csv',
  set: (value) => {
    if (!config_v2.value)
      return
    config_v2.value.export_path = value
    sync()
  },
})

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
