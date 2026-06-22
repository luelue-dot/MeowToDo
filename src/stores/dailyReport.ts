import { defineStore } from 'pinia'
import { ref } from 'vue'

import type { DailyReport, ReportQuery, ReportListResult } from '@/services/ai'
import { dailyReportService } from '@/services/ai'

export const useDailyReportStore = defineStore('dailyReport', () => {
  const reports = ref<DailyReport[]>([])
  const total = ref(0)
  const loading = ref(false)
  const generating = ref(false)

  const fetchReports = async (query: ReportQuery = {}): Promise<void> => {
    loading.value = true
    try {
      const result: ReportListResult = await dailyReportService.list(query)
      reports.value = result.reports
      total.value = result.total
    } finally {
      loading.value = false
    }
  }

  const generateReport = async (): Promise<DailyReport> => {
    generating.value = true
    try {
      const report = await dailyReportService.generate()
      // Refresh the list
      await fetchReports()
      return report
    } finally {
      generating.value = false
    }
  }

  return {
    reports,
    total,
    loading,
    generating,
    fetchReports,
    generateReport,
  }
})
