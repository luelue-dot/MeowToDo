<script setup lang="ts">
import { Checkbox, Input, message, Modal, Select } from "antdv-next"
import { computed, onMounted, onUnmounted, ref } from "vue"

import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"
import { useTodoStore, type Todo } from "@/stores/todo"

const todoStore = useTodoStore()
const newTaskTitle = ref("")
const newTaskPriority = ref<number>(0)
const createModalVisible = ref(false)

const sortedTasks = computed(() => {
  return [...todoStore.todayTasks].sort((a, b) => {
    const priorityRank = (p: number | null | undefined): number => {
      if (p === 1) return 0
      if (p === 2) return 1
      return 2
    }
    const pa = priorityRank(a.priority ?? 0)
    const pb = priorityRank(b.priority ?? 0)
    if (pa !== pb) return pa - pb
    const ta = a.created_at ?? ""
    const tb = b.created_at ?? ""
    return ta.localeCompare(tb)
  })
})

const pendingTasks = computed(() =>
  sortedTasks.value.filter((t) => t.status !== "completed"),
)

const completionMessages = ["太棒了！", "任务完成！", "继续保持！"]

function showRandomMessage() {
  const msg = completionMessages[Math.floor(Math.random() * completionMessages.length)]
  message.success(msg)
}

async function handleCompleteTask(todo: Todo) {
  await todoStore.completeTask(todo.id)
  showRandomMessage()
}

async function handleDeleteTask(id: number) {
  await todoStore.deleteTask(id)
}

async function handleCreateTask() {
  if (!newTaskTitle.value.trim()) return
  await todoStore.createTask({
    title: newTaskTitle.value.trim(),
    priority: newTaskPriority.value,
  })
  newTaskTitle.value = ""
  newTaskPriority.value = 0
  createModalVisible.value = false
  message.success("任务已创建")
}

const priorityText = (p: number | null | undefined): string => {
  if (p === 1) return "高"
  if (p === 2) return "中"
  return "低"
}

const priorityBadgeColor = (p: number | null | undefined): string => {
  if (p === 1) return "#ef4444"
  if (p === 2) return "#eab308"
  return "#9ca3af"
}

const statusEmoji = (todo: Todo): string => {
  if (todo.status === "completed") return "\u2705"  // ✅
  const p = todo.priority ?? 0
  if (p === 1) return "\U0001F525"  // 🔥
  if (p === 2) return "\u2B50"      // ⭐
  return "\U0001F31D"               // 🌝
}

const refreshData = () => {
  todoStore.fetchTodayTasks()
}

import { emit } from '@tauri-apps/api/event'

const dropdownEl = ref<HTMLElement>()

onMounted(() => {
  refreshData()
})

onMounted(() => {
  // Emit events when mouse enters/leaves the dropdown window
  const el = dropdownEl.value
  if (!el) return
  const onEnter = () => { emit('dropdown-mouse-enter', true) }
  const onLeave = () => { emit('dropdown-mouse-leave', true) }
  el.addEventListener('mouseenter', onEnter)
  el.addEventListener('mouseleave', onLeave)
})

const refreshTimer = setInterval(refreshData, 30000)

onUnmounted(() => {
  clearInterval(refreshTimer)
})
</script>

<template>
  <div ref="dropdownEl" class="dropdown-window">
    <div class="dropdown-header" data-tauri-drag-region>
      📋 今日任务
      <span class="task-count">({{ pendingTasks.length }})</span>
      <div class="spacer" />
      <div class="add-btn i-lucide:plus" @click="createModalVisible = true" />
    </div>

    <div class="dropdown-list">
      <div
        v-for="todo in sortedTasks"
        :key="todo.id"
        class="dropdown-item"
        :class="{ dimmed: todo.status === 'completed' }"
      >
        <Checkbox
          :checked="todo.status === 'completed'"
          @change="todo.status !== 'completed' && handleCompleteTask(todo)"
        />
        <span
          class="dropdown-item-title"
          :class="{ strikethrough: todo.status === 'completed' }"
        >
          {{ todo.title }}
        </span>
        <span
          v-if="todo.status !== 'completed'"
          class="priority-badge"
          :style="{ background: priorityBadgeColor(todo.priority) }"
        >
          {{ priorityText(todo.priority) }}
        </span>
        <div class="delete-btn i-lucide:trash-2" @click="handleDeleteTask(todo.id)" />
      </div>
      <div v-if="sortedTasks.length === 0" class="empty-hint">暂无任务</div>
    </div>

    <Modal
      v-model:open="createModalVisible"
      title="新增任务"
      :footer="null"
      width="360px"
      :destroy-on-close="true"
    >
      <div class="create-modal-body">
        <div class="create-field">
          <div class="create-label">任务标题</div>
          <Input
            v-model:value="newTaskTitle"
            placeholder="请输入任务标题"
            @pressEnter="handleCreateTask"
          />
        </div>
        <div class="create-field">
          <div class="create-label">优先级</div>
          <Select
            v-model:value="newTaskPriority"
            style="width: 100%"
            :options="[
              { value: 0, label: '低' },
              { value: 1, label: '高' },
              { value: 2, label: '中' },
            ]"
          />
        </div>
        <div class="flex justify-end mt-2">
          <button
            class="create-btn"
            :disabled="!newTaskTitle.trim()"
            @click="handleCreateTask"
          >
            确认
          </button>
        </div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.dropdown-window {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: rgba(20, 20, 30, 0.95);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  color: #eee;
  overflow: hidden;
}
.dropdown-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 14px;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  cursor: grab;
  -webkit-app-region: drag;
  app-region: drag;
}
.spacer { flex: 1; }
.task-count {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.45);
  font-weight: 400;
}
.add-btn {
  font-size: 18px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: color 0.15s, transform 0.15s;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}
.add-btn:hover { color: #fff; transform: scale(1.2); }
.dropdown-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 8px;
}
.dropdown-list::-webkit-scrollbar { width: 4px; }
.dropdown-list::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.15); border-radius: 2px; }
.dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 6px;
  transition: background 0.15s;
}
.dropdown-item:hover { background: rgba(255, 255, 255, 0.08); }
.dropdown-item.dimmed { opacity: 0.5; }
.dropdown-item-title {
  flex: 1;
  font-size: 13px;
  color: #eee;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.dropdown-item-title.strikethrough { color: rgba(255, 255, 255, 0.35); }
.priority-badge {
  font-size: 10px;
  color: #fff;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 600;
  flex-shrink: 0;
}
.delete-btn {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.3);
  cursor: pointer;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s;
}
.dropdown-item:hover .delete-btn { opacity: 1; }
.delete-btn:hover { color: #ef4444; }
.empty-hint {
  text-align: center;
  color: rgba(255, 255, 255, 0.35);
  font-size: 12px;
  padding: 32px 0;
}
.create-modal-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.create-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.create-label { font-size: 13px; color: #ccc; }
.create-btn {
  padding: 5px 20px;
  background: #1677ff;
  color: #fff;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: opacity 0.15s;
}
.create-btn:hover { opacity: 0.8; }
.create-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>