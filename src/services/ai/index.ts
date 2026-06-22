import { invoke } from '@tauri-apps/api/core'

export interface AiSettings {
  id: number
  provider: string
  model_name: string
  api_key: string
  base_url: string
  is_active: boolean
}

export interface SaveAiSettingsRequest {
  provider: string
  model_name: string
  api_key: string
  base_url: string
}

export interface GenerateTextRequest {
  prompt: string
}

export interface GenerateTextResponse {
  content: string
}

export interface DecomposeRequest {
  task_title: string
}

export interface SubtaskItem {
  title: string
  priority: number
}

export interface DecomposeResult {
  subtasks: SubtaskItem[]
}

/**
 * Unified AI service - all AI capabilities go through this module.
 */
export const aiService = {
  /** Get active AI settings */
  getSettings: async (): Promise<AiSettings> => {
    return await invoke<AiSettings>('get_ai_settings')
  },

  /** Save AI settings */
  saveSettings: async (request: SaveAiSettingsRequest): Promise<AiSettings> => {
    return await invoke<AiSettings>('save_ai_settings', { request })
  },

  /** Test AI connection with a simple prompt */
  testConnection: async (): Promise<string> => {
    return await invoke<string>('test_ai_connection')
  },

  /** Unified text generation */
  generateText: async (prompt: string): Promise<string> => {
    const result = await invoke<GenerateTextResponse>('invoke_llm', {
      request: { prompt },
    })
    return result.content
  },

  /** Decompose a task into subtasks with priorities */
  decomposeTask: async (taskTitle: string): Promise<SubtaskItem[]> => {
    const result = await invoke<DecomposeResult>('decompose_task', {
      task: { task_title: taskTitle },
    })
    return result.subtasks
  },
}

export interface DailyReport {
  id: number
  report_date: string
  content: string
  created_at: string | null
}

export interface ReportQuery {
  page?: number
  page_size?: number
  start_date?: string
  end_date?: string
  keyword?: string
}

export interface ReportListResult {
  reports: DailyReport[]
  total: number
}

/** Daily Report features */
export const dailyReportService = {
  /** Generate a new daily report using AI */
  generate: async (): Promise<DailyReport> => {
    const result = await invoke<{ report: DailyReport }>('generate_daily_report')
    return result.report
  },

  /** Get daily reports with pagination, date filter, and keyword search */
  list: async (query: ReportQuery = {}): Promise<ReportListResult> => {
    return await invoke<ReportListResult>('get_daily_reports', { query })
  },
}