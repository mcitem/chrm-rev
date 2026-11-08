<template>
  <div class="flex h-full min-h-0 flex-col pt-4">
    <Description :id="props.id" class="flex-shrink-0 px-5" />
    <Search :id="props.id" :add-item="itemId => changeQuantity(itemId, 1)" />

    <div
      ref="tableWrap"
      class="min-h-0 flex-1 flex-shrink overflow-hidden"
      style="background-color: var(--ant-color-bg-container)"
    >
      <a-table
        :columns="columns"
        :data-source="records.data.value?.records"
        :pagination="false"
        :scroll="{ y: height - 54, x: 600 }"
      >
        <template #emptyText>
          暂无数据
        </template>
      </a-table>
    </div>
    <div
      class="flex-shrink-0 px-3 py-4 text-right"
      style="background: var(--ant-color-bg-container)"
    >
      共 {{ records.data.value?.summary.category_count }} 种，
      {{ records.data.value?.summary.total_quantity }} 件，总计（折后价）
      {{ records.data.value?.summary.total_discount_price }} 元
    </div>
  </div>

  <a-modal
    :open="difficultyWarningOpen && tab === student?.student_no"
    centered
    title="贫困认定提醒"
    @cancel="difficultyWarningOpen = false"
  >
    <p>
      {{ student?.name }}（学号：{{
        student?.student_no
      }}）的认定级别为“不困难”，这位同学可能没有通过贫困认定，请核实后再办理登记。
    </p>
    <template #footer>
      <a-button type="primary" @click="difficultyWarningOpen = false">
        我已知晓
      </a-button>
    </template>
  </a-modal>

  <a-modal
    v-if="tab === student?.student_no"
    v-model:open="open"
    centered
    :closable="false"
    title="余额不足"
    cancel-text="不再提醒"
    @ok="
      () => {
        open = false
      }
    "
    @cancel="
      () => {
        open = false
        noOpen = true
      }
    "
  >
    <p>余额不足，请合理删减购买的商品</p>
    <template #footer="{ extra }">
      <component :is="extra.CancelBtn" />
      <a-button @click="() => instance.post('/sys/open_docs', {})">
        帮助
      </a-button>
      <component :is="extra.OkBtn" />
    </template>
  </a-modal>
</template>

<script setup lang="tsx">
import type { TableProps } from 'antdv-next'
import type { CSSProperties } from 'vue'
import type { Record } from '@/bindings/entity'
import type { DashboardHomeContext } from '@/lib/context'
import { DeleteOutlined, LineOutlined, PlusOutlined } from '@antdv-next/icons'
import { useElementSize } from '@vueuse/core'
import { DASHBOARD_HOME_CONTEXT_KEY } from '@/lib/context'
import { databaseChanging, databaseGeneration } from '@/lib/databaseSession'
import {
  instance,
  useStudent,
  useStudentRecordsWithSummary,
} from '@/lib/service'

import Description from './Description.vue'
import Search from './Search.vue'

const props = defineProps<{
  id: number
}>()

const { tab } = inject<DashboardHomeContext>(DASHBOARD_HOME_CONTEXT_KEY)!
const { data: student } = useStudent(props.id)

const difficultyWarningOpen = ref(false)
watch(
  () =>
    tab.value === student.value?.student_no
    && student.value?.difficulty_level === '不困难',
  (shouldWarn) => {
    difficultyWarningOpen.value = shouldWarn
  },
  { immediate: true },
)

const open = ref(false)
const noOpen = ref(false)
watch(
  () => student.value?.balance,
  (newVal, oldVal) => {
    if (newVal && oldVal) {
      const new_val = Number.parseFloat(newVal)
      const old_val = Number.parseFloat(oldVal)
      if (!noOpen.value && new_val < old_val && new_val < -5) {
        open.value = true
      }
    }
  },
)

const el = useTemplateRef('tableWrap')
const { height } = useElementSize(el)

const queryClient = useQueryClient()
const set_item = useMutation({
  mutationFn: (data: { item_id: number, quantity: number }) =>
    instance.post(`/biz/student/${props.id}/record/${data.item_id}`, {
      quantity: data.quantity,
    }),

  async onSuccess() {
    await queryClient.invalidateQueries({
      queryKey: ['biz', 'record', props.id],
    })
    await queryClient.invalidateQueries({
      queryKey: ['biz', 'student', props.id],
    })
    await queryClient.invalidateQueries({
      queryKey: ['biz', 'record', 'all_with_summary'],
    })
  },
})

const delete_item = useMutation({
  mutationFn: (item_id: number) =>
    instance.delete(`/biz/student/${props.id}/record/${item_id}`),
  async onSuccess() {
    await queryClient.invalidateQueries({
      queryKey: ['biz', 'record', props.id],
    })
    await queryClient.invalidateQueries({
      queryKey: ['biz', 'student', props.id],
    })
    await queryClient.invalidateQueries({
      queryKey: ['biz', 'record', 'all_with_summary'],
    })
  },
})

let disposed = false
onUnmounted(() => {
  disposed = true
})
let pendingQuantity = Promise.resolve()
function changeQuantity(itemId: number, delta: number | null) {
  const generation = databaseGeneration.value
  const active = () =>
    !disposed && !databaseChanging.value && generation === databaseGeneration.value
  pendingQuantity = pendingQuantity
    .then(async () => {
      if (!active())
        return
      const current = await queryClient.fetchQuery<Record[]>({
        queryKey: ['biz', 'record', props.id],
        queryFn: () => instance.get(`/biz/student/${props.id}/record`),
        staleTime: 0,
      })
      if (!active())
        return
      const existing = current.find(record => record.item_id === itemId)
      const quantity = delta === null ? 0 : (existing?.quantity ?? 0) + delta
      if (quantity <= 0) {
        if (existing)
          await delete_item.mutateAsync(itemId)
      }
      else {
        await set_item.mutateAsync({ item_id: itemId, quantity })
      }
    })
    .catch(() => {
      /* The response interceptor displays request errors. */
    })
  return pendingQuantity
}

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
    title: '商品名称',
    dataIndex: 'item_name',
    key: 'item_name',
    render: (text, record) => (
      <>
        <span style="color: var(--ant-color-text-secondary)">
          #
          {record.item_id}
        </span>
        {' '}
        {text}
      </>
    ),
    width: 200,
  },
  {
    title: '商品规格',
    dataIndex: 'item_spec',
    key: 'item_spec',
    width: 150,
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
    width: 90,
  },
  {
    title: '操作',
    key: 'action',
    render: (_, record) => (
      <>
        <AButton
          shape="circle"
          color="red"
          size="small"
          onClick={() => changeQuantity((record as Record).item_id, 1)}
          styles={
            {
              root: {
                border: 'none',
              },
            } satisfies {
              root: CSSProperties
            }
          }
        >
          {{
            icon: () => <PlusOutlined />,
          }}
        </AButton>
        <AButton
          shape="circle"
          color="red"
          size="small"
          onClick={() => changeQuantity((record as Record).item_id, -1)}
          styles={
            {
              root: {
                border: 'none',
              },
            } satisfies {
              root: CSSProperties
            }
          }
        >
          {{
            icon: () => <LineOutlined />,
          }}
        </AButton>
        <AButton
          shape="circle"
          color="red"
          size="small"
          onClick={() => changeQuantity((record as Record).item_id, null)}
          styles={
            {
              root: {
                border: 'none',
              },
            } satisfies {
              root: CSSProperties
            }
          }
        >
          {{
            icon: () => <DeleteOutlined />,
          }}
        </AButton>
      </>
    ),
  },
]

const records = useStudentRecordsWithSummary(props.id)
</script>

<style lang="css" scoped>
:deep(.ant-table-cell) {
  border-bottom: none !important;
}
</style>
