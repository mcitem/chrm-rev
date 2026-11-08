import type { QueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'
import { router } from '@/lib/router'
import { useStore } from '@/lib/stores'

export const databaseGeneration = ref(0)
export const databaseChanging = ref(false)

export function beginDatabaseChange() {
  databaseGeneration.value++
  databaseChanging.value = true
}

export function endDatabaseChange() {
  databaseChanging.value = false
}

export async function resetDatabaseSession(queryClient: QueryClient) {
  await queryClient.cancelQueries()
  useStore().clearTabs()
  queryClient.clear()
  await router.replace('/bootloader')
}
