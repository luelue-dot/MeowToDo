import codecs

path = 'E:/26cyf/01代码/BongoCat-master/src/components/task-panel/index.vue'
with codecs.open(path, 'r', encoding='utf-8-sig') as f:
    content = f.read()

# ========== SCRIPT SECTION: Add date-grouped logic ==========

# Add grouped tasks after the priorityColor function and before openDailyReportCenter
old_after_priority = '''async function openDailyReportCenter() {'
new_with_groups = '''// Group history tasks by date
const groupedByDate = computed(() => {
  const map = new Map<string, { date: string; pending: Todo[]; completed: Todo[] }>()
  
  // Add today section
  const today = new Date().toISOString().slice(0, 10)
  map.set(today, {
    date: today,
    pending: [...pendingTasks.value],
    completed: [...completedTodayTasks.value],
  })
  
  // Add historical dates
  for (const task of todoStore.historyTasks) {
    const d = task.task_date || task.created_at?.slice(0, 10)
    if (!d || d === today) continue
    if (!map.has(d)) {
      map.set(d, { date: d, pending: [], completed: [] })
    }
    const group = map.get(d)!
    if (task.status === 'completed') {
      if (!group.completed.find(t => t.id === task.id)) {
        group.completed.push(task)
      }
    } else {
      if (!group.pending.find(t => t.id === task.id)) {
        group.pending.push(task)
      }
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
  // Trigger reactivity
  expandedDates.value = new Set(expandedDates.value)
}

function formatDateLabel(dateStr: string): string {
  const today = new Date().toISOString().slice(0, 10)
  const yesterday = new Date(Date.now() - 86400000).toISOString().slice(0, 10)
  if (dateStr === today) return '今日'
  if (dateStr === yesterday) return '昨日'
  return dateStr
}

async function openDailyReportCenter() {'

content = content.replace(old_after_priority, new_with_groups)

# ========== TEMPLATE SECTION: Replace the task list ==========

# Replace the old task list template with date-grouped one
old_template_start = '''    <Flex vertical gap="small" class="flex-1 overflow-hidden px-2 pb-2">
      <div class="text-4 font-medium flex items-center gap-2">
        <span>{{ \('pages.main.todo.todayTasks') }}</span>
        <span class="text-sm text-gray-4">({{ pendingTasks.length }})</span>
      </div>

      <div v-if="pendingTasks.length === 0 && completedTodayTasks.length === 0" class="flex-1 flex items-center justify-center">
        <Empty :description="\('pages.main.todo.noTasks')" />
      </div>

      <div v-else class="flex-1 overflow-y-auto space-y-2">
        <div
          v-for="todo in pendingTasks"
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

        <div
          v-if="completedTodayTasks.length > 0"
          class="flex items-center gap-2 p-1.5 rounded cursor-pointer hover:bg-gray-5/10 transition select-none"
          @click="showCompleted = !showCompleted"
        >
          <div
            class="i-lucide:chevron-right transition"
            :class="{ 'rotate-90': showCompleted }"
          />
          <span>{{ \('pages.main.todo.completed') }} ({{ completedTodayTasks.length }})</span>
        </div>

        <div v-if="showCompleted" class="space-y-1 mt-1">
          <div
            v-for="todo in completedTodayTasks"
            :key="todo.id"
            class="flex items-center gap-2 p-1.5 rounded"
          >
            <Checkbox :checked="true" disabled />
            <span class="flex-1 text-sm line-through text-gray-4">{{ todo.title }}</span>
          </div>
        </div>
      </div>
    </Flex>'''

new_template = '''    <Flex vertical gap="small" class="flex-1 overflow-hidden px-2 pb-2">
      <div class="flex-1 overflow-y-auto space-y-1">
        <div v-if="groupedByDate.length === 0" class="flex-1 flex items-center justify-center h-full">
          <Empty :description="\('pages.main.todo.noTasks')" />
        </div>

        <template v-for="group in groupedByDate" :key="group.date">
          <div
            class="flex items-center gap-2 p-1.5 rounded cursor-pointer hover:bg-gray-5/10 transition select-none sticky top-0 z-10"
            style="background: rgba(26, 26, 46, 0.95); backdrop-filter: blur(8px)"
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
    </Flex>'''

content = content.replace(old_template_start, new_template)

with codecs.open(path, 'w', encoding='utf-8-sig') as f:
    f.write(content)
print('Updated todo panel with date-grouped view')
