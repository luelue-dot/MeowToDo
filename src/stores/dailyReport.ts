import { defineStore } from 'pinia'
import { ref } from 'vue'

import { dailyReportService, type DailyReport } from '@/services/ai'

export const useDailyReportStore = defineStore('dailyReport', () => {
  const currentReport = ref<DailyReport | null>(null)
  const loading = ref(false)

  const generateReport = async () => {
    loading.value = true
    try {
      currentReport.value = await dailyReportService.generate()
    } finally {
      loading.value = false
    }
  }

  return {
    currentReport,
    loading,
    generateReport,
  }
})