<template>
  <a-tabs
    v-model:active-key="tab"
    type="editable-card"
    hide-add
    :items="[MainTabDefinition, ...tabs]"
    :styles="{
      root: { height: '100%' },
      header: { gap: 0 },
      item: { borderRadius: 0, margin: 0 },
      content: {
        height: '100%',
      },
    }"
    @edit="
      (key, action) => {
        if (typeof key === 'string' && action === 'remove') {
          if (tab === key) {
            tab = MAIN_TAB_KEY
          }
          store.removeTab(key)
        }
      }
    "
  >
    <template #leftExtra>
      <LeftExtra />
    </template>
    <template #rightExtra>
      <RightExtra />
    </template>
  </a-tabs>
</template>

<script setup lang="tsx">
import type { DashboardHomeContext } from '@/lib/context'
import { storeToRefs } from 'pinia'
import { DASHBOARD_HOME_CONTEXT_KEY } from '@/lib/context'
import { useStore } from '@/lib/stores'
import LeftExtra from './LeftExtra.vue'
import RightExtra from './RightExtra.vue'
import StuTab from './StuTab/StuTab.vue'
import { MAIN_TAB_KEY, MainTabDefinition } from './tabs'

const store = useStore()

const tab = ref<string>(MAIN_TAB_KEY)
const { tabs: Rawtabs } = storeToRefs(store)

const tabs = computed(() => {
  return Rawtabs.value.map(student => ({
    key: student.student_no,
    label: student.name,
    content: <StuTab id={student.id} />,
  }))
})

provide<DashboardHomeContext>(DASHBOARD_HOME_CONTEXT_KEY, {
  tab,
  tabs,
})
</script>

<style scoped>
::v-deep(.ant-tabs-nav) {
  margin: 0;
}

::v-deep(.ant-tabs-content.ant-tabs-content-top) {
  height: 100%;
}

::v-deep(.ant-tabs-tab) {
  user-select: none;
  border: none;
}
</style>
