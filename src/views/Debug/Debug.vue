<template>
  <div
    class="h-screen w-screen bg-white text-black dark:bg-black dark:text-white"
  >
    <div>
      <a-button @click="useFetch">
        Fetch
      </a-button>

      <a-input v-model:value="method" placeholder="method" />

      <a-input v-model:value="url" placeholder="url" />

      <a-input v-model:value="body" placeholder="body" />
    </div>
    <div>
      {{ response_header }}
    </div>
    <div>
      {{ response }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { fetch } from 'tauri-plugin-axum-api/fetch'

const method = ref('GET')
const url = ref('https://axum.localhost/biz/student/1')
const response = ref('')
const response_header = ref('')
const body = ref('')

function useFetch() {
  fetch(url.value, {
    method: method.value,
    body: body.value ? body.value : undefined,
    headers: {
      'Content-Type': 'application/json',
    },
  })
    .then((res) => {
      response_header.value = JSON.stringify(res.status)
      return res.text()
    })
    .then((data) => {
      response.value = data
    })
    .catch((error) => {
      console.error(error)
      response.value = `Error: ${error}`
    })
}
</script>
