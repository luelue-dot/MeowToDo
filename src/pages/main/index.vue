<script setup lang="ts">

import type { MotionInfo } from "easy-live2d"

import { convertFileSrc } from "@tauri-apps/api/core"
import { PhysicalSize } from "@tauri-apps/api/dpi"
import { Menu, PredefinedMenuItem } from "@tauri-apps/api/menu"
import { sep } from "@tauri-apps/api/path"
import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow"
import { exists, readDir } from "@tauri-apps/plugin-fs"
import { useDebounceFn, useEventListener } from "@vueuse/core"
import { round } from "es-toolkit"
import { nth } from "es-toolkit/compat"
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue"

import { useAppMenu } from "@/composables/useAppMenu"
import { useDevice } from "@/composables/useDevice"
import { useGamepad } from "@/composables/useGamepad"
import { useModel } from "@/composables/useModel"
import { useTauriListen } from "@/composables/useTauriListen"
import { useTaskPosition } from "@/composables/useTaskPosition"
import { LISTEN_KEY, WINDOW_LABEL } from "@/constants"
import { setAlwaysOnTop, setTaskbarVisibility } from "@/plugins/window"
import { analyticsService } from '@/services/analytics'
import { useCatStore } from "@/stores/cat"
import { useGeneralStore } from "@/stores/general.ts"
import { useModelStore } from "@/stores/model"
import { useTodoStore } from "@/stores/todo"
import { isImage } from "@/utils/is"
import live2d from "@/utils/live2d"
import { join } from "@/utils/path"
import { isWindows } from "@/utils/platform"
import { clearObject } from "@/utils/shared"

const { startListening } = useDevice()
const appWindow = getCurrentWebviewWindow()
const { modelSize, handleLoad, handleDestroy, handleResize, handleKeyChange } = useModel()
const catStore = useCatStore()
const { getBaseMenu, getExitMenu } = useAppMenu()
const modelStore = useModelStore()
const generalStore = useGeneralStore()
const todoStore = useTodoStore()
const resizing = ref(false)
const backgroundImagePath = ref()
const { stickActive } = useGamepad()

const { edge, windowPos, compute: computeTaskPos } = useTaskPosition()
const HOVER_DELAY_MS = 200
let hoverTimer = null
let hideTimer = null
const pendingTasks = ref([])
const topPriorityTask = ref(null)

function updateTasks() {
  const all = todoStore.todayTasks
  const pending = all.filter(t => t.status !== "completed")
  pending.sort((a, b) => {
    const r = (p) => { if (p === 1) return 0; if (p === 2) return 1; return 2 }
    const pa = r(a.priority ?? 0); const pb = r(b.priority ?? 0)
    if (pa !== pb) return pa - pb
    return (a.created_at ?? "").localeCompare(b.created_at ?? "")
  })
  pendingTasks.value = pending
  topPriorityTask.value = pending[0] ?? null
}

function priorityText(p) {
  if (p === 1) return "高"
  if (p === 2) return "中"
  return "低"
}

function priorityColor(p) {
  if (p === 1) return "#ef4444"
  if (p === 2) return "#eab308"
  return "#9ca3af"
}

function priorityEmoji(p) {
  const v = p ?? 0
  if (v === 1) return "🔥"
  if (v === 2) return "⭐"
  return "🌝"
}

function showWindow() { appWindow.show() }
function hideWindow() { appWindow.hide() }

async function showTaskDropdown() {
  await computeTaskPos()
  const tdWindow = await WebviewWindow.getByLabel(WINDOW_LABEL.TASK_DROPDOWN)
  if (!tdWindow) return
  await tdWindow.setPosition(windowPos.value)
  await tdWindow.show()
  await tdWindow.unminimize()
  await tdWindow.setFocus()
}

async function hideTaskDropdown() {
  const tdWindow = await WebviewWindow.getByLabel(WINDOW_LABEL.TASK_DROPDOWN)
  if (tdWindow) await tdWindow.hide()
}

function handleTaskHoverEnter() {
  if (hideTimer) clearTimeout(hideTimer); hideTimer = null
  if (hoverTimer) clearTimeout(hoverTimer)
  hoverTimer = setTimeout(async () => {
    await showTaskDropdown()
  }, HOVER_DELAY_MS)
}

function handleTaskHoverLeave() {
  if (hoverTimer) clearTimeout(hoverTimer); hoverTimer = null
  // Give user 800ms to move to the dropdown before hiding
  hideTimer = setTimeout(hideTaskDropdown, 800)
}

watch(() => todoStore.todayTasks, updateTasks, { deep: true, immediate: true })

const refreshTimer = setInterval(() => todoStore.fetchTodayTasks(), 10000)

useTauriListen(LISTEN_KEY.START_MOTION, ({ payload }) => live2d.startMotion(payload))
useTauriListen(LISTEN_KEY.SET_EXPRESSION, ({ payload }) => live2d.setExpression(payload))
useTauriListen("task-completed", () => {
  // Trigger expression 2 on task completion
  // live2d.setExpression(2)
  live2d.startMotion({ group: 'CAT_motion', no: 1 })
})

useTauriListen("dropdown-mouse-enter", () => {
  if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
})

useTauriListen("dropdown-mouse-leave", () => {
  // Extend hide timer when leaving dropdown back to main window
  hideTimer = setTimeout(hideTaskDropdown, 800)
})

function handleMouseDown() { appWindow.startDragging() }

async function handleContextmenu(event) {
  event.preventDefault(); if (event.shiftKey) return
  const menu = await Menu.new({ items: [ ...await getBaseMenu(), await PredefinedMenuItem.new({ item: 'Separator' }), ...await getExitMenu() ] })
  if (isWindows && catStore.window.alwaysOnTop) setAlwaysOnTop(false)
  await menu.popup()
  if (isWindows && catStore.window.alwaysOnTop) setAlwaysOnTop(true)
}

function handleMouseMove(event) {
  if (event.buttons !== 2 || !event.shiftKey) return
  catStore.window.scale = round(Math.max(25, Math.min(catStore.window.scale + (event.movementX + event.movementY) * 0.5, 500)))
}

onMounted(startListening)
onUnmounted(() => { handleDestroy(); clearInterval(refreshTimer) })

watch(() => modelStore.currentModel, async (model) => {
  if (!model) return; await handleLoad()
  const p = join(model.path, "resources", "background.png")
  backgroundImagePath.value = (await exists(p)) ? convertFileSrc(p) : void 0
  clearObject([modelStore.supportKeys, modelStore.pressedKeys])
  for await (const g of ['left-keys', 'right-keys']) {
    const d = join(model.path, "resources", g)
    const files = (await readDir(d).catch(() => [])).filter(f => isImage(f.name))
    for (const f of files) modelStore.supportKeys[f.name.split(".")[0]] = join(d, f.name)
  }
  modelStore.modelReady = true
}, { deep: true, immediate: true })

watch([() => catStore.window.scale, modelSize], async ([s, ms]) => { if (!ms) return; appWindow.setSize(new PhysicalSize({ width: Math.round(ms.width * (s / 100)), height: Math.round(ms.height * (s / 100)) })) }, { immediate: true })

watch([modelStore.pressedKeys, stickActive], ([keys, sa]) => {
  const ds = Object.values(keys).map(p => { const s = p.split(sep()); return s[s.length - 2] })
  handleKeyChange(true, sa.left || ds.some(d => d.startsWith("left")))
  handleKeyChange(false, sa.right || ds.some(d => d.startsWith("right")))
}, { deep: true })

watch(() => catStore.window.visible, v => v ? showWindow() : hideWindow())
watch(() => catStore.window.passThrough, v => appWindow.setIgnoreCursorEvents(v), { immediate: true })
watch(() => catStore.window.alwaysOnTop, setAlwaysOnTop, { immediate: true })
watch(() => generalStore.app.taskbarVisible, setTaskbarVisibility, { immediate: true })
watch(() => catStore.model.motionSound, live2d.setMotionSoundEnabled, { immediate: true })
watch(() => catStore.model.maxFPS, live2d.setMaxFPS, { immediate: true })

watch(() => catStore.window.radius, async (val) => {
  await nextTick(); const el = document.querySelector(".size-screen")
  if (!el) return; el.style.borderRadius = val === 0 ? "0.01%" : "0%"
  el.offsetHeight; el.style.borderRadius = val + "%"
})

const debouncedResize = useDebounceFn(async () => { await handleResize(); resizing.value = false }, 100)
useEventListener("resize", () => { resizing.value = true; debouncedResize() })
</script>

<template>
  <div class="relative size-screen" :class="{ '-scale-x-100': catStore.model.mirror }" :style="{ borderRadius: catStore.window.radius + '%' }" @contextmenu="handleContextmenu" @mousedown="handleMouseDown" @mousemove="handleMouseMove">
    <div class="cat-content-layer absolute inset-0 overflow-hidden children:(absolute size-full)" :style="{ opacity: catStore.window.opacity / 100 }">
      <img v-if="backgroundImagePath" class="object-cover" :src="backgroundImagePath">
      <canvas id="live2dCanvas" />
      <img v-for="path in modelStore.pressedKeys" :key="path" class="object-cover" :src="convertFileSrc(path)">
      <div v-show="resizing || !modelStore.modelReady" class="flex items-center justify-center bg-black">
                <span class="text-center text-[10vw] text-[#fff]">{{ resizing ? $t('pages.main.hints.redrawing') : $t('pages.main.hints.switching') }}</span>
      </div>
    </div>
    <div class="collapsed-task-bar" @mouseenter="handleTaskHoverEnter" @mouseleave="handleTaskHoverLeave">
      <div class="task-bar-content">
        <template v-if="topPriorityTask">
          <span class="emoji">{{ priorityEmoji(topPriorityTask.priority) }}</span>
          <span class="task-title">{{ topPriorityTask.title }}</span>
          <span class="priority-label" :style="{ background: priorityColor(topPriorityTask.priority) }">{{ priorityText(topPriorityTask.priority) }}</span>
        </template>
        <template v-else>
          <span class="dim-text">暂无任务</span>
        </template>
        <span class="chevron"></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.collapsed-task-bar { position: absolute; bottom: 0; left: 0; right: 0; z-index: 10; display: flex; justify-content: center; pointer-events: auto; }
.task-bar-content { background: rgba(20, 20, 30, 0.85); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); padding: 6px 14px; cursor: pointer; white-space: nowrap; display: flex; align-items: center; gap: 8px; font-size: 13px; color: #fff; border-radius: 10px 10px 0 0; border-top: 1px solid rgba(255,255,255,0.12); max-width: 100%; width: auto; min-width: 120px; transition: background 0.25s ease; }
.task-bar-content:hover { background: rgba(20, 20, 30, 0.95); }
.task-title { overflow: hidden; text-overflow: ellipsis; max-width: 140px; }
.dim-text { color: rgba(255,255,255,0.5); font-size: 13px; }
.emoji { flex-shrink: 0; }
.chevron { flex-shrink: 0; width: 0; height: 0; border-left: 5px solid transparent; border-right: 5px solid transparent; border-top: 6px solid rgba(255,255,255,0.5); margin-left: 4px; }
.priority-label { font-size: 10px; color: #fff; padding: 1px 6px; border-radius: 4px; font-weight: 600; flex-shrink: 0; }
</style>
