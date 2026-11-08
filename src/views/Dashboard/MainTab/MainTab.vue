<template>
  <div class="flex h-full min-h-0 flex-col pt-5">
    <Header />

    <Search />

    <div
      class="flex flex-shrink-0 flex-grow-0 items-center gap-2 px-3 pb-2 pt-3"
    >
      <a-button type="dashed" :loading="openingCsv" :disabled="exportPathModalOpen" @click="openCsv">
        <template #icon>
          <LinkOutlined />
        </template>
        电子表
      </a-button>
      <Export />
    </div>

    <Table />

    <Footer />
    <ExportPathModal
      v-model:open="exportPathModalOpen"
      hint="尚未配置可用的电子表，请选择已有文件或创建新文件；选好后会自动保存路径并打开。"
      :saving="savingExportPath"
      @selected="saveExportPath"
    />
  </div>
</template>

<script setup lang="tsx">
import { LinkOutlined } from '@antdv-next/icons'
import { App } from 'antdv-next'
import ExportPathModal from '@/components/ExportPathModal.vue'
import { instance } from '@/lib/service'
import Export from './Export.vue'
import Footer from './Footer.vue'
import Header from './Header.vue'
import Search from './Search.vue'
import Table from './Table.vue'

const { message } = App.useApp()
const queryClient = useQueryClient()
const openingCsv = ref(false)
const exportPathModalOpen = ref(false)
const savingExportPath = ref(false)

async function openCsv() {
  if (openingCsv.value)
    return
  openingCsv.value = true
  try {
    const result = (await instance.post('/sys/open_csv')) as { needs_configuration: boolean }
    exportPathModalOpen.value = result.needs_configuration
  }
  catch {
    // The shared response interceptor displays the backend's opening error.
  }
  finally {
    openingCsv.value = false
  }
}

async function saveExportPath(path: string) {
  if (savingExportPath.value)
    return
  savingExportPath.value = true
  try {
    await instance.put('/sys/export_file/path', { path })
  }
  catch {
    return
  }
  finally {
    savingExportPath.value = false
  }
  exportPathModalOpen.value = false
  message.success('导出路径已保存')
  await queryClient.invalidateQueries({ queryKey: ['/sys/config'] })
  await openCsv()
}
</script>

<style lang="css" scoped>
:deep(.ant-table-cell) {
  border-bottom: none !important;
}
</style>
