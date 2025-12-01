import type { RouteRecordRaw } from 'vue-router'
import { createRouter, createWebHistory } from 'vue-router'

const routeRecord: RouteRecordRaw[] = [
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
