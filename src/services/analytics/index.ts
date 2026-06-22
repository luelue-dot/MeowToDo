import { invoke } from '@tauri-apps/api/core'

export interface AnalyticsEvent {
  id: number
  event_name: string
  event_value: string | null
  created_at: string | null
}

export interface AnalyticsSummary {
  active_days: number
  tasks_created: number
  tasks_completed: number
  task_completion_rate: number
  ai_decompose_count: number
  ai_decompose_usage_rate: number
  recommendation_accept_rate: number
  daily_report_open_rate: number
}

export interface TrendData {
  dates: string[]
  tasks_created: number[]
  tasks_completed: number[]
  ai_decompose: number[]
  completion_rates: number[]
}

/**
 * Analytics service - handles event tracking, metrics computation, and report export.
 */
export const analyticsService = {
  /** Track an analytics event */
  trackEvent: async (eventName: string, eventValue?: string): Promise<void> => {
    try {
      await invoke('track_event_command', {
        eventName,
        eventValue: eventValue ?? null,
      })
    } catch (e) {
      console.warn('Failed to track analytics event:', e)
    }
  },

  /** Get analytics summary */
  getSummary: async (): Promise<AnalyticsSummary> => {
    return invoke<AnalyticsSummary>('get_analytics_summary')
  },

  /** Get trend data for charts */
  getTrend: async (days: number = 7): Promise<TrendData> => {
    return invoke<TrendData>('get_analytics_trend', { days })
  },

  /** Get recent analytics events */
  getEvents: async (limit: number = 50): Promise<AnalyticsEvent[]> => {
    return invoke<AnalyticsEvent[]>('get_analytics_events', { limit })
  },

  /** Get testing mode status */
  getTestingMode: async (): Promise<boolean> => {
    return invoke<boolean>('get_testing_mode')
  },

  /** Set testing mode */
  setTestingMode: async (enabled: boolean): Promise<void> => {
    return invoke('set_testing_mode', { enabled })
  },

  /** Export analytics report as JSON string */
  exportReport: async (): Promise<string> => {
    return invoke<string>('export_analytics_report')
  },
}
