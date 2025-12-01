<template>
  <div v-if="loading" class="loading" />
</template>

<script setup lang="ts">
import type { BootloaderContextInner } from '@/bindings/bootloader'

const loading = ref(true)

const router = useRouter()

useQuery({
  queryKey: ['bootloader'],
  queryFn: async () => {
    const res = (await instance.get(
      '/sys/bootloader/context',
    )) as BootloaderContextInner
    console.log(res)
    const { conf_ready, db_ready } = res

    if (!conf_ready && !db_ready) {
      router.push({
        path: '/backupmanager',
      })
    }
    else if (!conf_ready) {
      router.push({
        path: '/configloader',
      })
    }
    else if (!db_ready) {
      router.push({
        path: '/dataloader',
      })
    }
    else {
      router.push({
        path: '/dashboard',
      })
    }

    return res
  },
})

// onMounted(() => {
// bootloader.refetch()
// })
</script>
