<template>
  <div class="mx-40 flex-shrink-0 pt-2">
    <a-auto-complete
      v-model:value="Params.q"
      class="w-full"
      placeholder="搜索学生"
      size="large"
      allow-clear
      :options="options"
      not-found-content="无匹配结果"
      :styles="
        tab !== MAIN_TAB_KEY || Params.q === ''
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
import type { Student } from '@/bindings/entity'
import type { OrderPagination } from '@/bindings/utils'
import type { DashboardHomeContext } from '@/lib/context'
import { DASHBOARD_HOME_CONTEXT_KEY } from '@/lib/context'
import { useStudents } from '@/lib/service'
import { useStore } from '@/lib/stores'
import { MAIN_TAB_KEY } from '../tabs'

const { tab } = inject<DashboardHomeContext>(DASHBOARD_HOME_CONTEXT_KEY)!

const store = useStore()

const Params = reactive<OrderPagination>({
  page: 1,
  per_page: 5,
  order_by_key: null,
  order_by_type: null,
  q: '',
})

interface Option {
  origin: Student
  value: string
  label: JSX.Element
}

function onSelect(value: string, option: Option) {
  store.pubTab(option.origin)
  tab.value = value
}

const StuQuery = useStudents(Params)

const options = computed(() => {
  return Params.q
    ? StuQuery.data.value?.data?.map(
        s =>
          ({
            origin: s,
            value: s.student_no,
            label: <>{`${s.name} - ${s.student_no} - ${s.difficulty_level}`}</>,
          }) satisfies Option,
      )
    : []
})
</script>
