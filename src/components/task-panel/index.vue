<script setup lang="ts">
import { Checkbox, Empty, Flex, Input, message, Modal, Select, Spin } from 'antdv-next'
import { computed, onMounted, ref } from 'vue'

import { analyticsService } from '@/services/analytics'
import { useTodoStore, type CreateTodoRequest, type Todo } from '@/stores/todo'
import type { SubtaskItem } from '@/services/ai'

const todoStore = useTodoStore()

const newTaskTitle = ref('')
const newTaskPriority = ref<number>(0)
const showCompleted = ref(false)
const createModalVisible = ref(false)

// AI Task Decomposition
const decomposeModalVisible = ref(false)
const decomposing = ref(false)
const aiRawResponse = ref('')
const subtasks = ref<SubtaskItem[]>([])
const editedSubtasks = ref<SubtaskItem[]>([])

const pendingTasks = computed(() =>
  todoStore.todayTasks.filter(t => t.status !== 'completed'),
)

const completedTodayTasks = computed(() =>
  todoStore.todayTasks.filter(t => t.status === 'completed'),
)

const completionMessages = ['太棒了！', '任务完成！', '继续保持！']

function showRandomMessage() {
  const msg = completionMessages[Math.floor(Math.random() * completionMessages.length)]
  message.success(msg)
}

async function handleCreateTask() {
  if (!newTaskTitle.value.trim()) return

  const task: CreateTodoRequest = {
    title: newTaskTitle.value.trim(),
    priority: newTaskPriority.value,
  }

  await todoStore.createTask(task)
  newTaskTitle.value = ''
  newTaskPriority.value = 0
  createModalVisible.value = false
  message.success('任务已创建')
}

async function handleCompleteTask(todo: Todo) {
  await todoStore.completeTask(todo.id)
  showRandomMessage()
}

async function handleDeleteTask(id: number) {
  await todoStore.deleteTask(id)
}

// AI Decomposition
async function handleAIDecompose() {
  if (!newTaskTitle.value.trim()) {
    message.warning('请先输入任务标题')
    return
  }

  decomposing.value = true
  aiRawResponse.value = ''
  try {
    const result = await todoStore.decomposeTask(newTaskTitle.value.trim())
    analyticsService.trackEvent('ai_decompose', newTaskTitle.value.trim()).catch(() => {})
    subtasks.value = result
    editedSubtasks.value = result.map(s => ({ ...s }))
    analyticsService.trackEvent('recommendation_generated').catch(() => {})
    decomposeModalVisible.value = true
  } catch (e: any) {
    message.error('AI 拆解失败：' + (e?.toString() || '未知错误'))
  } finally {
    decomposing.value = false
  }
}

function removeSubtask(index: number) {
  editedSubtasks.value.splice(index, 1)
}

function addSubtask() {
  editedSubtasks.value.push({ title: '', priority: 0 })
}

function updateSubtaskTitle(index: number, value: string) {
  editedSubtasks.value[index].title = value
}

function updateSubtaskPriority(index: number, value: number) {
  editedSubtasks.value[index].priority = value
}

async function handleCreateAllSubtasks() {
  const validSubtasks = editedSubtasks.value.filter(s => s.title.trim())
  if (validSubtasks.length === 0) {
    message.warning('没有有效的子任务')
    return
  }

  for (const subtask of validSubtasks) {
    await todoStore.createTask({
      title: subtask.title.trim(),
      priority: subtask.priority,
    })
  }

  analyticsService.trackEvent('recommendation_accept', subtask.title.trim()).catch(() => {})
    decomposeModalVisible.value = false
    createModalVisible.value = false
    newTaskTitle.value = ''
  newTaskPriority.value = 0
  message.success(`已创建 ${validSubtasks.length} 个子任务`)
}

const priorityText = (p?: number | null): string => {
  if (p === 1) return '高'
  if (p === 2) return '中'
  return '低'
}

const priorityColor = (p?: number | null): string => {
  if (p === 1) return 'text-red-5'
  if (p === 2) return 'text-yellow-5'
  return 'text-gray-4'
}

// Group history tasks by date
const groupedByDate = computed(() => {
  const map = new Map<string, { date: string; pending: Todo[]; completed: Todo[] }>()
  const today = new Date().toISOString().slice(0, 10)
  map.set(today, {
    date: today,
    pending: [...pendingTasks.value],
    completed: [...completedTodayTasks.value],
  })
  for (const task of todoStore.historyTasks) {
    const d = task.task_date || task.created_at?.slice(0, 10)
    if (!d || d === today) continue
    if (!map.has(d)) {
      map.set(d, { date: d, pending: [], completed: [] })
    }
    const group = map.get(d)!
    if (task.status === 'completed') {
      if (!group.completed.find(t => t.id === task.id)) group.completed.push(task)
    } else {
      if (!group.pending.find(t => t.id === task.id)) group.pending.push(task)
    }
  }
  return Array.from(map.values()).sort((a, b) => b.date.localeCompare(a.date))
})
const expandedDates = ref<Set<string>>(new Set([new Date().toISOString().slice(0, 10)]))
function toggleDateGroup(date: string) {
  if (expandedDates.value.has(date)) {
    expandedDates.value.delete(date)
  } else {
    expandedDates.value.add(date)
  }
  expandedDates.value = new Set(expandedDates.value)
}
function formatDateLabel(dateStr: string): string {
  const today = new Date().toISOString().slice(0, 10)
  const yesterday = new Date(Date.now() - 86400000).toISOString().slice(0, 10)
  if (dateStr === today) return '今日'
  if (dateStr === yesterday) return '昨日'
  return dateStr
}
async function openDailyReportCenter() {
  const { showWindow } = await import('@/plugins/window')
  const { WINDOW_LABEL } = await import('@/constants')
  showWindow(WINDOW_LABEL.DAILY_REPORT)
}

onMounted(() => {
  todoStore.fetchTodayTasks()
  todoStore.fetchHistoryTasks()
})
</script>

<template>
  <div class="flex flex-col gap-4 h-full">
    <div class="flex items-center justify-between px-2 pt-2">
      <h2 class="text-5 font-bold m-0 flex items-center gap-2"><span class="i-lucide:list-todo text-5" />{{ $t('pages.main.todo.title') }}</h2>
      <div class="flex items-center gap-2">
        <div
          class="i-lucide:file-text cursor-pointer text-4 hover:text-primary transition"
          title="日报中心"
          @click="openDailyReportCenter"
        />
        <div
          class="i-lucide:plus cursor-pointer text-5 hover:text-primary transition"
          @click="createModalVisible = true"
        />
      </div>
    </div>

    <Flex vertical gap="small" class="flex-1 overflow-hidden px-2 pb-2">
      <div class="flex-1 overflow-y-auto space-y-1">
        <div v-if="groupedByDate.length === 0" class="flex-1 flex items-center justify-center h-full">
          <Empty :description="('pages.main.todo.noTasks')" />
        </div>

        <template v-for="group in groupedByDate" :key="group.date">
          <div
            class="flex items-center gap-2 p-1.5 rounded cursor-pointer hover:bg-gray-5/10 transition select-none sticky top-0 z-10"
            style="background: rgba(200, 200, 210, 0.25); backdrop-filter: blur(8px)"
            @click="toggleDateGroup(group.date)"
          >
            <div
              class="i-lucide:chevron-right text-3 transition"
              :class="{ 'rotate-90': expandedDates.has(group.date) }"
            />
            <span class="text-4 font-medium">{{ formatDateLabel(group.date) }}</span>
            <span class="text-xs text-gray-4">
              待办{{ group.pending.length }} / 完成{{ group.completed.length }}
            </span>
          </div>

          <div v-if="expandedDates.has(group.date)" class="space-y-1 pl-2">
            <!-- Pending tasks -->
            <div
              v-for="todo in group.pending"
              :key="todo.id"
              class="flex items-center gap-2 p-2 rounded-lg hover:bg-gray-5/10 transition group"
            >
              <Checkbox @change="handleCompleteTask(todo)" />
              <span class="flex-1 text-sm truncate">{{ todo.title }}</span>
              <span
                class="text-xs px-1.5 py-0.5 rounded font-medium"
                :class="priorityColor(todo.priority)"
              >{{ priorityText(todo.priority) }}</span>
              <div
                class="i-lucide:trash-2 cursor-pointer text-4 text-gray-4 hover:text-red-5 transition opacity-0 group-hover:opacity-100"
                @click="handleDeleteTask(todo.id)"
              />
            </div>

            <!-- Completed tasks -->
            <div v-if="group.completed.length > 0" class="mt-1">
              <div
                v-for="todo in group.completed"
                :key="todo.id"
                class="flex items-center gap-2 p-1.5 rounded opacity-60"
              >
                <Checkbox :checked="true" disabled />
                <span class="flex-1 text-sm line-through text-gray-4">{{ todo.title }}</span>
              </div>
            </div>
          </div>
        </template>
      </div>
    </Flex>

    <!-- Create Task Modal -->
    <Modal
      v-model:open="createModalVisible"
      :title="$t('pages.main.todo.createTask')"
      :footer="null"
      width="420px"
      :destroy-on-close="true"
    >
      <Flex vertical gap="middle">
        <div>
          <div class="text-sm mb-1">{{ $t('pages.main.todo.taskTitle') }}</div>
          <Input
            v-model:value="newTaskTitle"
            :placeholder="$t('pages.main.todo.taskTitlePlaceholder')"
            @pressEnter="handleCreateTask"
          />
        </div>

        <div>
          <div class="text-sm mb-1">{{ $t('pages.main.todo.priority') }}</div>
          <Select
            v-model:value="newTaskPriority"
            style="width: 100%"
            :options="[
              { value: 0, label: $t('pages.main.todo.priorityLow') },
              { value: 1, label: $t('pages.main.todo.priorityHigh') },
              { value: 2, label: $t('pages.main.todo.priorityMedium') },
            ]"
          />
        </div>

        <div class="flex justify-between items-center">
          <button
            class="px-4 py-1.5 bg-purple-600 text-black rounded-lg text-sm hover:opacity-80 transition disabled:opacity-40 flex items-center gap-1"
            :disabled="!newTaskTitle.trim() || decomposing"
            @click="handleAIDecompose"
          >
            <Spin v-if="decomposing" class="w-3.5 h-3.5" />
            <span v-else class="i-lucide:sparkles text-sm" />
            <span>{{ decomposing ? '拆解中...' : 'AI 拆解任务' }}</span>
          </button>
          <button
            class="px-4 py-1.5 bg-primary text-white rounded-lg text-sm hover:opacity-80 transition disabled:opacity-40"
            :disabled="!newTaskTitle.trim()"
            @click="handleCreateTask"
          >
            {{ $t('pages.main.todo.confirm') }}
          </button>
        </div>
      </Flex>
    </Modal>

    <!-- AI Decomposition Result Modal -->
    <Modal
      v-model:open="decomposeModalVisible"
      title="AI 拆解结果"
      :footer="null"
      width="580px"
      :destroy-on-close="true"
    >
      <div class="text-sm text-gray-500 mb-3">
        AI 已将「{{ newTaskTitle }}」拆解为以下子任务，你可以编辑标题、优先级，或增删：
      </div>

      <div class="space-y-2 max-h-80 overflow-y-auto">
        <div
          v-for="(subtask, index) in editedSubtasks"
          :key="index"
          class="flex items-center gap-2 p-2 rounded-lg hover:bg-gray-5/10 transition group border border-gray-5/20"
        >
          <span class="text-sm text-gray-400 w-5 flex-shrink-0">{{ index + 1 }}.</span>
          <Input
            :value="subtask.title"
            @input="(e: any) => updateSubtaskTitle(index, e.target.value)"
            class="flex-1 min-w-0"
            size="small"
            placeholder="任务标题"
          />
          <Select
            :value="subtask.priority"
            @update:value="(v: number) => updateSubtaskPriority(index, v)"
            style="width: 80px; flex-shrink: 0"
            size="small"
            :options="[
              { value: 0, label: '低' },
              { value: 2, label: '中' },
              { value: 1, label: '高' },
            ]"
          />
          <div
            class="i-lucide:x cursor-pointer text-4 text-gray-4 hover:text-red-5 transition flex-shrink-0"
            @click="removeSubtask(index)"
          />
        </div>
      </div>

      <div class="flex items-center gap-2 mt-2">
        <div
          class="i-lucide:plus-circle cursor-pointer text-lg text-primary hover:opacity-80 transition"
          @click="addSubtask"
        />
        <span class="text-xs text-gray-4 cursor-pointer hover:text-primary transition" @click="addSubtask">
          添加子任务
        </span>
      </div>

      <div class="flex justify-end mt-4">
        <button
          class="px-4 py-1.5 bg-primary text-white rounded-lg text-sm hover:opacity-80 transition"
          @click="handleCreateAllSubtasks"
        >
          创建所有任务
        </button>
      </div>
    </Modal>
  </div>
</template>

