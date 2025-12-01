import type { AxiosError as AxiosErrorRaw } from 'axios'
import type { Reactive } from 'vue'
import type { ConfigInner } from '@/bindings/config_v2'
import type { Item, Record, Student } from '@/bindings/entity'
import type {
  AllRecordWithSummary,
  StudentRecordWithSummary,
} from '@/bindings/summary'
import type { OrderPagination, PaginateData } from '@/bindings/utils'
import { keepPreviousData } from '@tanstack/vue-query'
import axios from 'axios'

export type AxiosError = AxiosErrorRaw<
  {
    msg?: string
    error?: string
  },
  unknown
>

export const instance = axios.create({
  baseURL: 'https://axum.localhost/',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

/// 只能被调用一次
export class RegisterInterceptor {
  static #instance: RegisterInterceptor | null = null
  constructor(messageErrorFn?: (msg: unknown) => void) {
    if (RegisterInterceptor.#instance) {
      return RegisterInterceptor.#instance
    }
    console.log('Registering interceptor...')

    RegisterInterceptor.#instance = this

    instance.interceptors.response.use(
      (response) => {
        if (
          response.status === 200
          && response.headers['content-type'] === 'application/json'
          && response.data
        ) {
          return response.data
        }
        return response
      },
      (error: AxiosError) => {
        console.error(error.response?.data.error)
        if (error.response?.data.msg) {
          console.error(error.response.data.msg)
          messageErrorFn?.(error.response.data.msg)
        }
        if (
          error.response?.headers['content-type']
          === 'text/plain; charset=utf-8'
          && typeof error.response.data === 'string'
        ) {
          console.error(error.response.data)
          messageErrorFn?.(error.response.data)
        }
        return Promise.reject(error)
      },
    )
  }
}

export function useConfig() {
  return useQuery<ConfigInner>({
    queryKey: ['/sys/config'],
    placeholderData: keepPreviousData,
    queryFn: () => instance.get('/sys/config'),
  })
}

export function useSysTime() {
  return useQuery<{
    time: string
    year: number
    date: string
  }>({
    queryKey: ['/sys/time'],
    queryFn: () => instance.get('/sys/time'),
    // 在dev模式下低频更新，避免干扰调试
    refetchInterval: import.meta.env.DEV ? 10000 : 1000,
  })
}

export function useStudents(
  Params: OrderPagination | Reactive<OrderPagination>,
) {
  return useQuery<PaginateData<Student[]>>({
    queryKey: ['biz', 'student', 'list', Params],
    placeholderData: keepPreviousData,
    queryFn: () =>
      instance.get('/biz/student/list', {
        params: Params,
      }),
  })
}

export function useItems(Params: OrderPagination | Reactive<OrderPagination>) {
  return useQuery<PaginateData<Item[]>>({
    queryKey: ['biz', 'item', 'list', Params],
    placeholderData: keepPreviousData,
    queryFn: () =>
      instance.get('/biz/item/list', {
        params: Params,
      }),
  })
}

export function useStudent(id: number) {
  return useQuery<Student>({
    queryKey: ['biz', 'student', id],
    placeholderData: keepPreviousData,
    queryFn: () => instance.get(`/biz/student/${id}`),
  })
}

export function useAllRecordsWithSummary() {
  return useQuery<AllRecordWithSummary>({
    queryKey: ['biz', 'record', 'all_with_summary'],
    queryFn: () => instance.get(`/biz/record/all_with_summary`),
  })
}

export function useStudentRecords(id: number) {
  return useQuery<Record[]>({
    queryKey: ['biz', 'record', id],
    placeholderData: keepPreviousData,
    queryFn: () => instance.get(`/biz/student/${id}/record`),
  })
}

export function useStudentRecordsWithSummary(id: number) {
  return useQuery<StudentRecordWithSummary>({
    queryKey: ['biz', 'record', id, 'with_summary'],
    placeholderData: keepPreviousData,
    queryFn: () => instance.get(`/biz/student/${id}/record_with_summary`),
  })
}

export const DB_DIRTY = 'System(Dataloader(DbDirty))'
