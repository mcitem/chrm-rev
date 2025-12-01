<template>
  <div
    class="ant-tabs-tab"
    :style="{ borderRadius: 0, margin: 0 }"
    @click="
      () => {
        instance.post('/sys/open_timedate_cpl')
      }
    "
  >
    <span> <ClockCircleOutlined /> {{ sysTime.data.value?.time }} </span>
  </div>
</template>

<script setup lang="ts">
import { ClockCircleOutlined } from '@antdv-next/icons'
import { instance, useSysTime } from '@/lib/service'

const sysTime = useSysTime()
</script>
