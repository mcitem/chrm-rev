<template>
  <div class="flex-shrink-0 px-5 pt-4">
    <a-auto-complete
      v-model:value="Params.q"
      class="w-full"
      size="large"
      :options="options"
      not-found-content="无数据"
      placeholder="请输入商品名称、商品编号"
      :styles="
        tab !== student?.student_no || Params.q === ''
          ? {
            popup: {
              root: {
                display: 'none',
              },
            },
          }
          : {}
      "
      @select="onSelect"
    />
  </div>
</template>

<script setup lang="tsx">
import type { JSX } from 'vue/jsx-runtime'
import type { Item } from '@/bindings/entity'
import type { OrderPagination } from '@/bindings/utils'
import type { DashboardHomeContext } from '@/lib/context'
import { DASHBOARD_HOME_CONTEXT_KEY } from '@/lib/context'
import { useItems, useStudent, useStudentRecords } from '@/lib/service'

const props = defineProps<{
  id: number

  mutateFn: (data: { item_id: number, quantity: number }) => void
}>()

const { tab } = inject<DashboardHomeContext>(DASHBOARD_HOME_CONTEXT_KEY)!

const { data: student } = useStudent(props.id)

const Params = reactive<OrderPagination>({
  page: 1,
  per_page: 5,
  order_by_key: null,
  order_by_type: null,
  q: '',
})

const records = useStudentRecords(props.id)

interface Option {
  value: number
  label: JSX.Element
}

function onSelect(
  item_id: number,
  _: {
    origin: Item
    value: number
    label: JSX.Element
  },
) {
  props.mutateFn({
    item_id,
    quantity:
      (records.data.value?.find(r => r.item_id === item_id)?.quantity || 0) + 1,
  })
  nextTick(() => {
    Params.q = ''
  })
}

const ItemQuery = useItems(Params)

const options = computed(() => {
  return Params.q
    ? ItemQuery.data.value?.data?.map(
        i =>
          ({
            value: i.id,
            label: (
              <>
                #
                {i.id}
                {' '}
                -
                {' '}
                {i.name}
                {' '}
                -
                {' '}
                {i.spec}
              </>
            ),
          }) satisfies Option,
      )
    : []
})

watch(
  () => Params.q,
  (newVal) => {
    if (typeof newVal === 'string' && newVal.startsWith('0') && newVal.length === 3) {
      const id = Number.parseInt(newVal)
      if (!Number.isNaN(id) && id !== 0) {
        props.mutateFn({
          item_id: id,
          quantity:
            (records.data.value?.find(r => r.item_id === id)?.quantity || 0) + 1,
        })

        Params.q = ''
      }
    }
  },
)
</script>
