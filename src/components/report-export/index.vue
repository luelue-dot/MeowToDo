<script setup lang="ts">
import { Button, message, Modal, Space } from 'antdv-next'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { ref } from 'vue'

import { analyticsService, type AnalyticsSummary } from '@/services/analytics'

const visible = ref(false)
const exporting = ref(false)

function open() {
  visible.value = true
}

async function exportJSON() {
  exporting.value = true
  try {
    const jsonStr = await analyticsService.exportReport()
    const filePath = await save({
      defaultPath: 'usage_report.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (filePath) {
      await writeTextFile(filePath, jsonStr)
      message.success('Report exported as JSON')
    }
  } catch (e: any) {
    message.error(`Export failed: ${e}`)
  } finally {
    exporting.value = false
  }
}

async function exportCSV() {
  exporting.value = true
  try {
    const summary: AnalyticsSummary = await analyticsService.getSummary()
    const now = new Date()
    const dateStr = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}`
    const headers = 'Metric,Value\n'
    const rows = [
      `Active Days,${summary.active_days}`,
      `Tasks Created,${summary.tasks_created}`,
      `Tasks Completed,${summary.tasks_completed}`,
      `Task Completion Rate (%),${summary.task_completion_rate}`,
      `AI Decompose Count,${summary.ai_decompose_count}`,
      `AI Decompose Usage Rate (%),${summary.ai_decompose_usage_rate}`,
      `Recommendation Accept Rate (%),${summary.recommendation_accept_rate}`,
      `Daily Report Open Rate (%),${summary.daily_report_open_rate}`,
    ].join('\n')
    const csv = headers + rows
    const filePath = await save({
      defaultPath: `usage_report_${dateStr}.csv`,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    })
    if (filePath) {
      await writeTextFile(filePath, csv)
      message.success('Report exported as CSV')
    }
  } catch (e: any) {
    message.error(`Export failed: ${e}`)
  } finally {
    exporting.value = false
  }
}

defineExpose({ open })
</script>

<template>
  <Modal v-model:open="visible" :title="$t('pages.analytics.exportReport')" :footer="null" width="400">
    <p class="mb-4 text-text-secondary">
      {{ $t('pages.analytics.exportHint') }}
    </p>
    <Space>
      <Button :loading="exporting" type="primary" @click="exportJSON">
        Export JSON
      </Button>
      <Button :loading="exporting" @click="exportCSV">
        Export CSV
      </Button>
    </Space>
  </Modal>
</template>
