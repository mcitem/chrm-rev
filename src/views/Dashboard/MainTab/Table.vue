<template>
  <div
    ref="tableWrap"
    class="flex-1 flex-shrink overflow-hidden"
    style="background-color: var(--ant-color-bg-container)"
  >
    <a-table
      :pagination="false"
      :data-source="records?.records"
      :columns="columns"
      :scroll="{ x: 800, y: height - 54 }"
    >
      <template #emptyText>
        暂无数据
      </template>
    </a-table>
  </div>
</template>

<script setup lang="tsx">
import type { TableProps } from 'antdv-next'
import { useElementSize } from '@vueuse/core'
import { useAllRecordsWithSummary } from '@/lib/service'

const el = useTemplateRef('tableWrap')
const { height } = useElementSize(el)

const { data: records } = useAllRecordsWithSummary()

const columns: TableProps['columns'] = [
  {
    title: '序号',
    key: 'no',
    render: (_, __, index) => (
      <>
        {' '}
        {index + 1}
      </>
    ),
    width: 60,
  },
  {
    title: '学号',
    dataIndex: 'student_no',
    key: 'student_no',
    width: 180,
  },
  {
    title: '认定级别',
    dataIndex: 'stu_d_level',
    key: 'stu_d_level',
    width: 100,
  },
  {
    title: '商品名称',
    dataIndex: 'item_name',
    key: 'item_name',
    width: 150,
  },
  {
    title: '商品规格',
    dataIndex: 'item_spec',
    key: 'item_spec',
    hidden: true,
    width: 100,
  },
  {
    title: '折后价',
    dataIndex: 'discount_price',
    key: 'discount_price',
    width: 90,
  },
  {
    title: '原价',
    dataIndex: 'original_price',
    key: 'original_price',
    width: 90,
  },
  {
    title: '数量',
    dataIndex: 'quantity',
    key: 'quantity',
  },
]
</script>
