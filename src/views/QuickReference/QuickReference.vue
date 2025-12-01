<template>
  <a-table
    class="h-screen overflow-y-auto"
    bordered
    sticky
    :pagination="false"
    :columns="columns"
    :row-hoverable="false"
    :row-class-name="
      (_, index) => {
        return index % 2 === 0 ? 'bg-gray-100 dark:bg-gray-900' : ''
      }
    "
    :data-source="items"
  />
</template>

<script setup lang="ts">
import type { TableProps } from 'antdv-next'
import type { Item } from '@/bindings/entity'
import { keepPreviousData } from '@tanstack/vue-query'
import { fetch } from 'tauri-plugin-axum-api/fetch'

function useAllItems() {
  return useQuery<Item[]>({
    queryKey: ['/biz/item/all'],
    placeholderData: keepPreviousData,
    queryFn: () => fetch('/biz/item/all').then(res => res.json()),
  })
}

const { data: items } = useAllItems()

const columns: TableProps['columns'] = [
  {
    title: '序号',
    dataIndex: 'id',
    key: 'id',
    width: 80,
  },
  {
    title: '商品名称',
    dataIndex: 'name',
    key: 'name',
    width: 200,
  },
  {
    title: '规格',
    dataIndex: 'spec',
    key: 'spec',
    width: 100,
  },
  {
    title: '原价',
    dataIndex: 'price',
    key: 'price',
  },
  {
    title: '特别困难',
    dataIndex: 'p_hard',
    key: 'p_hard',
  },
  {
    title: '困难',
    dataIndex: 'p_normal',
    key: 'p_normal',
  },
  {
    title: '一般困难',
    dataIndex: 'p_easy',
    key: 'p_easy',
  },
  {
    title: '积分',
    dataIndex: 'p_score',
    key: 'p_score',
  },
]
</script>
