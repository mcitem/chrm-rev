<template>
  <a-button
    type="default"
    @click="
      () => {
        open = true
      }
    "
  >
    导出
  </a-button>
  <a-modal
    centered
    :closable="false"
    title="确认导出"
    :open="open"
    :confirm-loading="confirmLoading"
    @ok="handleOk"
    @cancel="handleCancel"
  >
    <div>
      本班次金额（总折后价）：{{
        useAllRecords.data.value?.summary.total_discount_price ?? '0.00'
      }}
    </div>
    <div class="py-4">
      <div class="pb-2">
        <a-checkable-tag
          v-for="i in conf?.user_template"
          :key="i"
          class="select-none px-2 py-1 text-[16px]"
          :checked="checkedUsers[i]"
          @change="
            checkedValue => {
              checkedUsers[i] = checkedValue
            }
          "
        >
          {{ i }}
        </a-checkable-tag>
      </div>
      <div>
        <a-checkable-tag
          v-for="i in conf?.time_template"
          :key="i"
          class="mb-1 ml-2 select-none px-2 py-1.5 text-[16px]"
          :checked="checkedTime === i"
          @change="
            checkedValue => {
              if (checkedValue) {
                checkedTime = i
              }
            }
          "
        >
          {{ i }}
        </a-checkable-tag>
      </div>
      <a-input
        v-model:value="sign"
        size="large"
        placeholder="请输入签名"
        class="mt-4"
      />
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import type { ExportRequest } from '@/bindings/mutation'
import type { UnsafeInvokeVertify } from '@/bindings/sys'
import { App } from 'antdv-next'
import { useAllRecordsWithSummary, useConfig, useSysTime } from '@/lib/service'

const useAllRecords = useAllRecordsWithSummary()

const { message } = App.useApp()

const queryClient = useQueryClient()

const open = ref(false)
const confirmLoading = ref(false)

const { data: conf } = useConfig()
const sys_time = useSysTime()
const checkedTime = ref('')
const checkedUsers = ref<Record<string, boolean>>({})

const sign = ref('')

function syncSignString(
  checkedTimeValue: string,
  checkedUsersValue: Record<string, boolean>,
) {
  if (!sys_time.data.value)
    return
  sign.value = `${sys_time.data.value?.date} ${checkedTimeValue} ${Object.keys(
    checkedUsersValue,
  )
    .filter(key => checkedUsersValue[key])
    .join(' ')}`
}

watch(
  [checkedTime, checkedUsers],
  ([checkedTimeValue, checkedUsersValue]) => {
    syncSignString(checkedTimeValue, checkedUsersValue)
  },
  { immediate: true, deep: true },
)

const exportMutation = useMutation({
  mutationFn: async () => {
    confirmLoading.value = true

    await instance.post('/biz/record/export', {
      sign: sign.value,
    } satisfies ExportRequest)

    // console.log('export response', response)

    await instance.post('/sys/unsafe/clear_record', {
      secret: window.UNSAFE_INVOKE_SECRET,
    } satisfies UnsafeInvokeVertify)
  },
  onSuccess() {
    setTimeout(() => {
      open.value = false
      confirmLoading.value = false
      message.success('导出成功')
    }, 1000)

    queryClient.invalidateQueries()
  },
  onError() {
    message.error('导出失败')

    confirmLoading.value = false
  },
})

function handleOk() {
  if (confirmLoading.value)
    return
  exportMutation.mutate()
}

function handleCancel() {
  if (confirmLoading.value)
    return
  open.value = false
}
</script>
