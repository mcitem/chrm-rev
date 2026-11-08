<template>
  <a-modal
    v-model:open="openModal"
    title="配置导出电子表"
    :footer="null"
    :closable="!busy"
    :mask-closable="!busy"
    :keyboard="!busy"
  >
    <div class="flex flex-col gap-3 py-3">
      <div>
        <a-button :disabled="busy" @click="selectPath('existing')">
          选择已有电子表
        </a-button>
        <p class="pt-2">
          选择已有的 CSV 文件，之后的导出会追加到该文件。
        </p>
      </div>
      <div>
        <a-button type="primary" :disabled="busy" @click="selectPath('new')">
          选择文件夹并创建新电子表
        </a-button>
        <p class="pt-2">
          只需选择保存文件夹，程序自行创建文件
        </p>
      </div>
      <p>{{ hint }}</p>
      <p v-if="busy">
        正在处理，请稍候…
      </p>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { App } from 'antdv-next'
import { instance } from '@/lib/service'

const props = withDefaults(defineProps<{ hint: string, saving?: boolean }>(), { saving: false })
const emit = defineEmits<{ selected: [path: string] }>()
const openModal = defineModel<boolean>('open', { default: false })
const selecting = defineModel<boolean>('selecting', { default: false })
const busy = computed(() => selecting.value || props.saving)
const { message } = App.useApp()

async function selectPath(mode: 'existing' | 'new') {
  if (busy.value)
    return
  selecting.value = true
  try {
    const selected = await open(mode === 'existing'
      ? {
          title: '选择已有的导出电子表（CSV）',
          directory: false,
          multiple: false,
          filters: [{ name: 'CSV 电子表', extensions: ['csv'] }],
        }
      : { title: '选择新电子表的保存文件夹', directory: true, multiple: false })
    if (typeof selected !== 'string')
      return
    let path = selected
    if (mode === 'existing') {
      if (!selected.toLowerCase().endsWith('.csv')) {
        message.error('请选择 CSV 文件，Excel 工作簿请先另存为 CSV 格式')
        return
      }
    }
    else {
      const created = (await instance.post('/sys/export_file/create', { directory: selected })) as { path: string }
      path = created.path
    }
    emit('selected', path)
  }
  catch (error) {
    message.error(`配置导出路径失败：${error}`)
  }
  finally {
    selecting.value = false
  }
}
</script>
