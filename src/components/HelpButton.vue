<template>
  <a-button @click="() => instance.post(`/sys/open_docs`, { path })">
    帮助
  </a-button>
</template>

<script setup lang="ts">
import { instance } from '@/lib/service'

defineProps<{
  path?: string
}>()
</script>
