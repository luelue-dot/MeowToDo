import { analyticsService } from '@/services/analytics'
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { ref } from 'vue'

import { aiService, type SubtaskItem } from '@/services/ai'

export interface Todo {
  id: number
  title: string
  description?: string | null
  status?: string | null
  priority?: number | null
  deadline?: string | null
  task_date?: string | null
  created_at?: string | null
  completed_at?: string | null
  task_date?: string | null
}

export interface CreateTodoRequest {
  title: string
  description?: string | null
  status?: string | null
  priority?: number | null
  deadline?: string | null
  task_date?: string | null
}

export interface ActivityLog {
  id: number
  action: string
  task_id?: number | null
  timestamp?: string | null
}

export const useTodoStore = defineStore('todo', () => {
  const todayTasks = ref<Todo[]>([])
  const historyTasks = ref<Todo[]>([])
  const loading = ref(false)

  const createTask = async (task: CreateTodoRequest): Promise<Todo> => {
    const todo = await invoke<Todo>('create_task', { task })
    analyticsService.trackEvent('task_created', task.title).catch(() => {})
    await fetchTodayTasks()
    await fetchHistoryTasks()
    return todo
  }

  const updateTask = async (id: number, updates: Partial<Todo>): Promise<Todo> => {
    const todo = await invoke<Todo>('update_task', { task: { id, ...updates } })
    await fetchTodayTasks()
    await fetchHistoryTasks()
    return todo
  }

  const deleteTask = async (id: number): Promise<void> => {
    await invoke('delete_task', { id })
    analyticsService.trackEvent('task_deleted').catch(() => {})
    todayTasks.value = todayTasks.value.filter(t => t.id !== id)
    historyTasks.value = historyTasks.value.filter(t => t.id !== id)
  }

  const completeTask = async (id: number): Promise<Todo> => {
    const todo = await invoke<Todo>('complete_task', { id })
    analyticsService.trackEvent('task_completed').catch(() => {})
    await fetchTodayTasks()
    await fetchHistoryTasks()
    return todo
  }

  const decomposeTask = async (taskTitle: string): Promise<SubtaskItem[]> => {
    return await aiService.decomposeTask(taskTitle)
  }

  const fetchTodayTasks = async (): Promise<void> => {
    loading.value = true
    try {
      todayTasks.value = await invoke<Todo[]>('get_today_tasks')
    } finally {
      loading.value = false
    }
  }

  const fetchHistoryTasks = async (): Promise<void> => {
    loading.value = true
    try {
      historyTasks.value = await invoke<Todo[]>('get_task_history')
    } finally {
      loading.value = false
    }
  }

  return {
    todayTasks,
    historyTasks,
    loading,
    createTask,
    updateTask,
    deleteTask,
    completeTask,
    decomposeTask,
    fetchTodayTasks,
    fetchHistoryTasks,
  }
})