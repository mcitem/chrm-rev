import { ConfigInner } from '@/bindings/config';
import type { Item, Student } from '@/bindings/entity';
import type { OrderPagination, PaginateData } from '@/bindings/utils';
import { keepPreviousData, useQuery } from '@tanstack/vue-query';
import axios, { AxiosError } from 'axios';
import { toast } from 'vue-sonner';

export const instance = axios.create({
  baseURL: 'https://axum.localhost',
  timeout: 5000,
  headers: {
    'Content-Type': 'application/json',
  },
});

instance.interceptors.response.use(
  function (response) {
    if (
      response.status === 200 &&
      response.headers['content-type'] === 'application/json' &&
      response.data
    ) {
      // 如果返回200下的json，自动解出封装的data，不存在data字段则返回整个body
      return response.data.data ?? response.data;
    }
    // 其他情况直接返回原始axios response
    return response;
  },
  function (
    error: AxiosError<
      {
        msg?: string;
        error?: string;
      },
      unknown
    >
  ) {
    console.log(error);
    if (error.response?.data.msg) {
      toast.error(error.response.data.msg);
    }

    if (error.response?.data.error) {
      toast.error(error.response.data.error);
    }

    if (
      error.response?.headers['content-type'] === 'text/plain; charset=utf-8' &&
      typeof error.response.data === 'string'
    ) {
      toast.error(error.response.data);
    }

    return Promise.reject(error);
  }
);

export function GetStudents(Params: OrderPagination) {
  return useQuery<PaginateData<Student[]>>({
    queryKey: ['/student/list', Params],
    staleTime: 1000 * 60 * 5,
    placeholderData: keepPreviousData,
    queryFn: () =>
      instance.get('/student/list', {
        params: Params,
      }),
  });
}

export function GetItems(Params: OrderPagination) {
  return useQuery<PaginateData<Item[]>>({
    queryKey: ['/item/list', Params],
    staleTime: 1000 * 60 * 5,
    placeholderData: keepPreviousData,
    queryFn: () =>
      instance.get('/item/list', {
        params: Params,
      }),
  });
}

export function useConfig() {
  return useQuery<ConfigInner>({
    queryKey: ['/sys/config'],
    staleTime: 1000 * 60 * 5,
    placeholderData: keepPreviousData,
    queryFn: () => instance.get('/sys/config'),
  });
}
/**
 * time: %Y-%m-%d %H:%M:%S
 * date: 由配置决定,建议设置为 %Y-%m-%d
 */
export function useSysTime() {
  return useQuery<{
    time: string;
    year: number;
    date: string;
  }>({
    queryKey: ['/sys/time'],
    queryFn: () => instance.get('/sys/time'),
    // 在dev模式下不更新，避免干扰调试
    refetchInterval: import.meta.env.DEV ? false : 1000,
  });
}
