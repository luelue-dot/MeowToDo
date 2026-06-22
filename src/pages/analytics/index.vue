<script setup lang="ts">
import { Card, Flex, Spin, Statistic } from 'antdv-next'
import { onMounted, ref } from 'vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, BarChart } from 'echarts/charts'
import { TooltipComponent, LegendComponent, GridComponent, TitleComponent } from 'echarts/components'
import { useI18n } from 'vue-i18n'

import { analyticsService, type AnalyticsSummary, type TrendData } from '@/services/analytics'

use([CanvasRenderer, LineChart, BarChart, TooltipComponent, LegendComponent, GridComponent, TitleComponent])

const { t } = useI18n()
const loading = ref(true)
const summary = ref<AnalyticsSummary | null>(null)
const trend = ref<TrendData | null>(null)

const taskTrendOption = ref({})
const completionTrendOption = ref({})
const decomposeTrendOption = ref({})
const completionRateOption = ref({})

async function fetchData() {
  loading.value = true
  try {
    summary.value = await analyticsService.getSummary()
    trend.value = await analyticsService.getTrend(7)
    buildCharts()
  } finally {
    loading.value = false
  }
}

function buildCharts() {
  if (!trend.value) return

  const t = trend.value

  taskTrendOption.value = {
    tooltip: { trigger: 'axis' },
    grid: { left: 50, right: 20, bottom: 30, top: 10 },
    xAxis: { type: 'category', data: t.dates, axisLabel: { rotate: 30, fontSize: 10 } },
    yAxis: { type: 'value', minInterval: 1 },
    series: [{
      type: 'bar',
      data: t.tasks_created,
      itemStyle: { color: '#1677ff', borderRadius: [4, 4, 0, 0] },
      barMaxWidth: 32,
    }],
  }

  completionTrendOption.value = {
    tooltip: { trigger: 'axis' },
    grid: { left: 50, right: 20, bottom: 30, top: 10 },
    xAxis: { type: 'category', data: t.dates, axisLabel: { rotate: 30, fontSize: 10 } },
    yAxis: { type: 'value', minInterval: 1 },
    series: [{
      type: 'bar',
      data: t.tasks_completed,
      itemStyle: { color: '#52c41a', borderRadius: [4, 4, 0, 0] },
      barMaxWidth: 32,
    }],
  }

  decomposeTrendOption.value = {
    tooltip: { trigger: 'axis' },
    grid: { left: 50, right: 20, bottom: 30, top: 10 },
    xAxis: { type: 'category', data: t.dates, axisLabel: { rotate: 30, fontSize: 10 } },
    yAxis: { type: 'value', minInterval: 1 },
    series: [{
      type: 'line',
      data: t.ai_decompose,
      smooth: true,
      lineStyle: { color: '#722ed1', width: 2 },
      itemStyle: { color: '#722ed1' },
      areaStyle: { color: 'rgba(114,46,209,0.15)' },
    }],
  }

  completionRateOption.value = {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const p = params[0]
        return `${p.axisValue}<br/>${p.seriesName}: ${p.value}%`
      },
    },
    grid: { left: 50, right: 20, bottom: 30, top: 10 },
    xAxis: { type: 'category', data: t.dates, axisLabel: { rotate: 30, fontSize: 10 } },
    yAxis: { type: 'value', min: 0, max: 100, axisLabel: { formatter: '{value}%' } },
    series: [{
      name: t('pages.analytics.labels.completionRate'),
      type: 'line',
      data: t.completion_rates,
      smooth: true,
      lineStyle: { color: '#fa8c16', width: 2 },
      itemStyle: { color: '#fa8c16' },
      areaStyle: { color: 'rgba(250,140,22,0.15)' },
    }],
  }
}

onMounted(fetchData)
</script>

<template>
  <Spin :spinning="loading">
    <div class="p-6">
      <h2 class="mb-4 text-lg font-bold">{{ t('pages.analytics.title') }}</h2>

      <Flex wrap="wrap" gap="12">
        <Card class="flex-1 min-w-40" :title="t('pages.analytics.labels.dau')">
          <Statistic :value="summary?.active_days ?? 0" suffix="天" />
        </Card>
        <Card class="flex-1 min-w-40" :title="t('pages.analytics.labels.tasksCreated')">
          <Statistic :value="summary?.tasks_created ?? 0" />
        </Card>
        <Card class="flex-1 min-w-40" :title="t('pages.analytics.labels.tasksCompleted')">
          <Statistic :value="summary?.tasks_completed ?? 0" />
        </Card>
        <Card class="flex-1 min-w-40" :title="t('pages.analytics.labels.aiDecompose')">
          <Statistic :value="summary?.ai_decompose_count ?? 0" />
        </Card>
        <Card class="flex-1 min-w-40" :title="t('pages.analytics.labels.recommendAcceptRate')">
          <Statistic :value="summary?.recommendation_accept_rate ?? 0" suffix="%" :precision="1" />
        </Card>
        <Card class="flex-1 min-w-40" :title="t('pages.analytics.labels.dailyReportOpenRate')">
          <Statistic :value="summary?.daily_report_open_rate ?? 0" suffix="%" :precision="1" />
        </Card>
      </Flex>

      <Flex wrap="wrap" gap="12" class="mt-6">
        <Card class="flex-1 min-w-72" :title="t('pages.analytics.labels.taskCreateTrend')">
          <VChart class="h-56 w-full" :option="taskTrendOption" autoresize />
        </Card>
        <Card class="flex-1 min-w-72" :title="t('pages.analytics.labels.taskCompleteTrend')">
          <VChart class="h-56 w-full" :option="completionTrendOption" autoresize />
        </Card>
      </Flex>
      <Flex wrap="wrap" gap="12" class="mt-4">
        <Card class="flex-1 min-w-72" :title="t('pages.analytics.labels.aiDecomposeTrend')">
          <VChart class="h-56 w-full" :option="decomposeTrendOption" autoresize />
        </Card>
        <Card class="flex-1 min-w-72" :title="t('pages.analytics.labels.completionRateTrend')">
          <VChart class="h-56 w-full" :option="completionRateOption" autoresize />
        </Card>
      </Flex>
    </div>
  </Spin>
</template>
