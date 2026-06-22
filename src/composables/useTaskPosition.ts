import { PhysicalPosition } from "@tauri-apps/api/dpi"
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"
import { useRafFn } from "@vueuse/core"
import { ref } from "vue"

export type EdgePosition = "bottom" | "top" | "left" | "right"

export interface TaskPosition {
  edge: EdgePosition
  windowPosition: PhysicalPosition
  barPlacement: EdgePosition
}

const DROPDOWN_WIDTH = 340
const DROPDOWN_HEIGHT = 420
const GAP = 2

export function useTaskPosition() {
  const edge = ref<EdgePosition>("bottom")
  const windowPos = ref(new PhysicalPosition(0, 0))

  const compute = async () => {
    const appWindow = getCurrentWebviewWindow()
    const [winPos, winSize] = await Promise.all([
      appWindow.outerPosition(),
      appWindow.outerSize(),
    ])

    const { x, y } = winPos
    const { width: w, height: h } = winSize

    const spaceBottom = window.screen.availHeight - (y + h)
    const spaceTop = y
    const spaceRight = window.screen.availWidth - (x + w)
    const spaceLeft = x

    const maxSpace = Math.max(spaceBottom, spaceTop, spaceRight, spaceLeft)

    let currentEdge: EdgePosition
    let dropX: number
    let dropY: number

    if (maxSpace === spaceBottom || (spaceBottom >= DROPDOWN_HEIGHT && spaceBottom >= spaceTop && spaceBottom >= spaceRight && spaceBottom >= spaceLeft)) {
      currentEdge = "bottom"
      dropX = x + Math.round((w - DROPDOWN_WIDTH) / 2)
      dropY = y + h + GAP
    } else if (maxSpace === spaceTop || (spaceTop >= DROPDOWN_HEIGHT && spaceTop >= spaceRight && spaceTop >= spaceLeft)) {
      currentEdge = "top"
      dropX = x + Math.round((w - DROPDOWN_WIDTH) / 2)
      dropY = y - DROPDOWN_HEIGHT - GAP
    } else if (maxSpace === spaceRight || (spaceRight >= DROPDOWN_WIDTH && spaceRight >= spaceLeft)) {
      currentEdge = "right"
      dropX = x + w + GAP
      dropY = y + Math.round((h - DROPDOWN_HEIGHT) / 2)
    } else {
      currentEdge = "left"
      dropX = x - DROPDOWN_WIDTH - GAP
      dropY = y + Math.round((h - DROPDOWN_HEIGHT) / 2)
    }

    dropX = Math.max(0, Math.min(dropX, window.screen.availWidth - DROPDOWN_WIDTH))
    dropY = Math.max(0, Math.min(dropY, window.screen.availHeight - DROPDOWN_HEIGHT))

    edge.value = currentEdge
    windowPos.value = new PhysicalPosition(Math.round(dropX), Math.round(dropY))
  }

  const { pause, resume } = useRafFn(compute, { immediate: false })

  const startTracking = () => {
    compute()
    resume()
  }

  const stopTracking = () => {
    pause()
  }

  return {
    edge,
    windowPos,
    compute,
    startTracking,
    stopTracking,
  }
}