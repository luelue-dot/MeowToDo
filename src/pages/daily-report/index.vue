<script setup lang="ts">
import { Input, message, Modal, Pagination, Spin } from "antdv-next"
import { computed, onMounted, ref } from "vue"

import { analyticsService } from "@/services/analytics"
import { useDailyReportStore } from "@/stores/dailyReport"
import type { ReportQuery } from "@/services/ai"

const dailyReportStore = useDailyReportStore()

const keyword = ref("")
const startDate = ref("")
const endDate = ref("")
const currentPage = ref(1)
const pageSize = ref(10)
const previewReport = ref<typeof dailyReportStore.reports[0] | null>(null)
const previewVisible = ref(false)

const query = computed<ReportQuery>(() => ({
  page: currentPage.value,
  page_size: pageSize.value,
  start_date: startDate.value || undefined,
  end_date: endDate.value || undefined,
  keyword: keyword.value || undefined,
}))

const fetchData = async () => {
  await dailyReportStore.fetchReports(query.value)
}

const handleSearch = () => {
  currentPage.value = 1
  fetchData()
}

const handleGenerate = async () => {
  const confirmed = await new Promise<boolean>((resolve) => {
    Modal.confirm({
      title: "生成日报",
      content: "将根据今日任务完成情况由AI自动生成日报，是否继续？",
      onOk: () => resolve(true),
      onCancel: () => resolve(false),
    })
  })
  if (!confirmed) return

  try {
    const report = await dailyReportStore.generateReport()
    analyticsService.trackEvent("daily_report_generated").catch(() => {})
    message.success("日报生成成功！")
    previewReport.value = report
    previewVisible.value = true
  } catch (e: any) {
    message.error(e || "生成失败，请检查AI配置")
  }
}

const handlePreview = (report: typeof dailyReportStore.reports[0]) => {
  analyticsService.trackEvent("daily_report_open").catch(() => {})
  previewReport.value = report
  previewVisible.value = true
}

const formatDate = (dateStr: string) => {
  if (!dateStr) return ""
  const d = new Date(dateStr)
  return d.toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  })
}

onMounted(() => {
  fetchData()
})
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1>日报中心</h1>
      <button class="generate-btn" :disabled="dailyReportStore.generating" @click="handleGenerate">
        {{ dailyReportStore.generating ? "生成中..." : "生成今日日报" }}
      </button>
    </div>

    <div class="filter-bar">
      <Input v-model:value="keyword" placeholder="搜索日报内容..." style="width:200px" allow-clear @pressEnter="handleSearch" />
      <Input v-model:value="startDate" placeholder="开始日期 YYYY-MM-DD" style="width:160px" />
      <Input v-model:value="endDate" placeholder="结束日期 YYYY-MM-DD" style="width:160px" />
      <button class="btn btn-primary" @click="handleSearch">搜索</button>
      <button class="btn btn-default" @click="keyword=''; startDate=''; endDate=''; currentPage=1; fetchData()">重置</button>
    </div>

    <div class="report-list">
      <Spin :spinning="dailyReportStore.loading">
        <div v-if="dailyReportStore.reports.length === 0 && !dailyReportStore.loading" class="empty-hint">
          暂无日报数据
        </div>
        <div v-for="report in dailyReportStore.reports" :key="report.id" class="report-card" @click="handlePreview(report)">
          <div class="card-header">
            <span class="card-date">{{ report.report_date }}</span>
            <span class="card-time">{{ formatDate(report.created_at ?? '') }}</span>
          </div>
          <div class="card-content">{{ report.content.slice(0, 120) }}{{ report.content.length > 120 ? '...' : '' }}</div>
        </div>
      </Spin>
    </div>

    <div v-if="dailyReportStore.total > pageSize" class="pagination-bar">
      <Pagination v-model:current="currentPage" :page-size="pageSize" :total="dailyReportStore.total" @change="fetchData" />
    </div>

    <Modal v-model:open="previewVisible" :title="previewReport ? '日报 - ' + previewReport.report_date : ''" :footer="null" width="640px">
      <div v-if="previewReport" style="padding:8px 0">
        <div style="font-size:13px;color:#1677ff;font-weight:600;margin-bottom:12px">{{ previewReport.report_date }}</div>
        <div style="font-size:14px;line-height:1.8;color:#333;white-space:pre-wrap">{{ previewReport.content }}</div>
        <div style="font-size:12px;color:#999;border-top:1px solid #f0f0f0;padding-top:12px;margin-top:16px">
          生成时间：{{ formatDate(previewReport.created_at ?? '') }}
        </div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.page-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 20px 24px;
  background: #f5f5f5;
  color: #333;
  font-size: 14px;
  overflow: hidden;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.page-header h1 {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
  color: #222;
}

.generate-btn {
  padding: 6px 16px;
  background: linear-gradient(135deg, #1677ff, #6366f1);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
}
.generate-btn:hover { opacity: 0.85; }
.generate-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.filter-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.btn {
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  border: 1px solid #d9d9d9;
}
.btn-primary { background: #1677ff; color: #fff; border-color: #1677ff; }
.btn-primary:hover { opacity: 0.85; }
.btn-default { background: #fff; color: #333; }
.btn-default:hover { color: #1677ff; border-color: #1677ff; }

.report-list {
  flex: 1;
  overflow-y: auto;
}

.empty-hint {
  text-align: center;
  color: #999;
  font-size: 14px;
  padding: 80px 0;
}

.report-card {
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 8px;
  padding: 14px 16px;
  margin-bottom: 10px;
  cursor: pointer;
  transition: all 0.2s;
}
.report-card:hover {
  border-color: #d9d9d9;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.card-date { font-size: 14px; font-weight: 600; color: #222; }
.card-time { font-size: 11px; color: #999; }
.card-content { font-size: 13px; line-height: 1.6; color: #555; }

.pagination-bar {
  display: flex;
  justify-content: center;
  padding: 12px 0 0;
}
</style>
