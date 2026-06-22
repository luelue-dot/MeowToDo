import { invoke } from '@tauri-apps/api/core'

export interface AnalyticsEvent {
  id?: number
  event_name: string
  event_value?: string | null
  created_at?: string | null
}

export const analyticsService = {
  trackEvent: async (eventName: string, eventValue?: string): Promise<void> => {
    try {
      await invoke('track_analytics_event', {
        event: {
          event_name: eventName,
          event_value: eventValue ?? null,
        },
      })
    } catch (e) {
      console.warn('Failed to track analytics event:', e)
    }
  },
}