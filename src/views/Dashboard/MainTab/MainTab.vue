<template>
  <div class="flex h-full flex-col pt-5">
    <Header />

    <Search />

    <div
      class="flex flex-shrink-0 flex-grow-0 items-center gap-2 px-3 pb-2 pt-3"
    >
      <a-button type="dashed" @click="() => instance.post('/sys/open_csv')">
        <template #icon>
          <LinkOutlined />
        </template>
        电子表
      </a-button>
      <Export />
    </div>

    <Table />

    <Footer />
  </div>
</template>

<script setup lang="tsx">
import { LinkOutlined } from '@antdv-next/icons'
import { instance } from '@/lib/service'
import Export from './Export.vue'
import Footer from './Footer.vue'
import Header from './Header.vue'
import Search from './Search.vue'
import Table from './Table.vue'
</script>

<style lang="css" scoped>
:deep(.ant-table-cell) {
  border-bottom: none !important;
}
</style>
