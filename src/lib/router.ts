import type { RouteRecordRaw } from 'vue-router'
import { message } from 'antdv-next'
import { createRouter, createWebHistory } from 'vue-router'
import { instance } from '@/lib/service'

const routeRecord: RouteRecordRaw[] = [
  {
    path: '/datamanager',
    name: 'DataManager',
    component: () => import('@/views/DataManager/DataManager.vue'),
    beforeEnter: async () => {
      try {
        const records = await instance.get('/biz/record/all')
        if (!Array.isArray(records))
          throw new Error('Invalid records response')
        if (records.length) {
          message.warning('首页还有购买记录，不允许管理。')
          return '/dashboard'
        }
        return true
      }
      catch {
        message.error('无法检查首页记录，请稍后重试。')
        return '/dashboard'
      }
    },
  },
  {
    path: '/',
    redirect: '/bootloader',
  },
  {
    path: '/bootloader',
    name: 'Bootloader',
    component: () => import('@/views/Bootloader/Bootloader.vue'),
  },
  {
    path: '/backupmanager',
    name: 'BackupManager',
    component: () => import('@/views/BackupManager/BackupManager.vue'),
  },
  {
    path: '/configloader',
    name: 'Configloader',
    component: () => import('@/views/Configloader/Configloader.vue'),
  },
  {
    path: '/dataloader',
    name: 'Dataloader',
    component: () => import('@/views/Dataloader/Dataloader.vue'),
  },
  {
    path: '/dashboard',
    name: 'Dashboard',
    component: () => import('@/views/Dashboard/DashBoard.vue'),
  },
  {
    path: '/QuickReference',
    name: 'QuickReference',
    component: () => import('@/views/QuickReference/QuickReference.vue'),
  },
  {
    path: '/debug',
    name: 'Debug',
    component: () => import('@/views/Debug/Debug.vue'),
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    redirect: '/',
  },
]

export const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: routeRecord,
})

export default router

// {
//   path: '/init',
//   name: 'Init',
//   component: () => import('@/views/Init.vue'),
// },
// {
//   path: '/dashboard',
//   name: 'Dashboard',
//   component: () => import('@/views/DashBoard.vue'),
//   children: [
//     {
//       path: 'index',
//       name: 'DashboardHome',
//       component: () => import('@/views/DashboardHome.vue'),
//     },
//     {
//       path: 'control',
//       name: 'DashBoardControl',
//       component: () => import('@/views/DashBoardControl.vue'),
//     },
//     {
//       path: 'settings',
//       name: 'DashboardSettings',
//       component: () => import('@/views/DashboardSettings.vue'),
//     },
//   ] satisfies (RouteRecordRaw & {
//     name: DsahBoardRouteName;
//     component: Component;
//   })[],
// },
