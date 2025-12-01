/// <reference types="vite/client" />
import type { AxiosError } from './service'
import '@tanstack/vue-query'

declare global {
  interface Window {
    UNSAFE_INVOKE_SECRET: string
  }
}

declare module '@tanstack/vue-query' {
  interface Register {
    defaultError: AxiosError
  }
}
