import type { RouteRecordRaw } from 'vue-router'

import { createRouter, createWebHashHistory } from 'vue-router'

import Main from '../pages/main/index.vue'
import Preference from '../pages/preference/index.vue'

const routes: Readonly<RouteRecordRaw[]> = [
  {
    path: '/',
    component: Main,
  },
  {
    path: '/preference',
    component: Preference,
  },
  {
    path: '/task-dropdown',
    component: () => import('../pages/task-dropdown/index.vue'),
  },
  {
    path: '/todo',
    component: () => import('../pages/todo/index.vue'),
  },
  {
    path: '/daily-report',
    component: () => import('../pages/daily-report/index.vue'),
  },
  {
    path: '/analytics',
    component: () => import('../pages/analytics/index.vue'),
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router