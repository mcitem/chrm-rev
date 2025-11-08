import DashBoard from '@/views/DashBoard.vue';
import DashBoardControl from '@/views/DashBoardControl.vue';
import DashboardHome from '@/views/DashboardHome.vue';
import DashboardSettings from '@/views/DashboardSettings.vue';
import Init from '@/views/Init.vue';
import { Component } from 'vue';
import {
  createRouter,
  createWebHistory,
  type RouteRecordRaw,
} from 'vue-router';

/// 侧边栏路由
export const DashBoardRoutes = [
  {
    name: 'DashboardHome',
    title: '首页',
    icon: 'mdi-home',
  },
  {
    name: 'DashBoardControl',
    title: '管理',
    icon: 'mdi-table',
  },
  {
    name: 'DashboardSettings',
    title: '设置',
    icon: 'mdi-cog',
  },
] as const;

type DsahBoardRouteName = (typeof DashBoardRoutes)[number]['name'];

// 路由同步导入，后续不在加载
const routeRecord: RouteRecordRaw[] = [
  {
    path: '/',
    // 不能是tsx组件
    redirect: '/init',
  },
  {
    path: '/init',
    name: 'Init',
    component: Init,
  },
  {
    path: '/dashboard',
    name: 'Dashboard',
    component: DashBoard,
    children: [
      {
        path: 'index',
        name: 'DashboardHome',
        component: DashboardHome,
      },
      {
        path: 'control',
        name: 'DashBoardControl',
        component: DashBoardControl,
      },
      {
        path: 'settings',
        name: 'DashboardSettings',
        component: DashboardSettings,
      },
    ] satisfies (RouteRecordRaw & {
      name: DsahBoardRouteName;
      // 由于keepalive存在，必须同步加载component
      component: Component;
    })[],
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    redirect: '/',
  },
];

export const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: routeRecord,
});

export default router;
