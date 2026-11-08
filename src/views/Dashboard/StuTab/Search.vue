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
import { useItems, useStudent } from '@/lib/service'

const props = defineProps<{
  id: number

  addItem: (itemId: number) => void
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
  props.addItem(item_id)
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
    if (
      typeof newVal === 'string'
      && newVal.startsWith('0')
      && newVal.length === 3
    ) {
      const id = Number.parseInt(newVal)
      if (!Number.isNaN(id) && id !== 0) {
        props.addItem(id)

        Params.q = ''
      }
    }
  },
)
</script>
