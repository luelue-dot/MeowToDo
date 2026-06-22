use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: i64,
    pub event_name: String,
    pub event_value: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct UserMetrics {
    pub id: i64,
    pub metric_date: Option<String>,
    pub dau: i64,
    pub tasks_created: i64,
    pub tasks_completed: i64,
    pub ai_decompose_count: i64,
    pub recommendation_accept_count: i64,
    pub recommendation_total_count: i64,
    pub daily_report_open_count: i64,
    pub daily_report_generated_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub active_days: i64,
    pub tasks_created: i64,
    pub tasks_completed: i64,
    pub task_completion_rate: f64,
    pub ai_decompose_count: i64,
    pub ai_decompose_usage_rate: f64,
    pub recommendation_accept_rate: f64,
    pub daily_report_open_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DailyTrend {
    pub date: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub dates: Vec<String>,
    pub tasks_created: Vec<i64>,
    pub tasks_completed: Vec<i64>,
    pub ai_decompose: Vec<i64>,
    pub completion_rates: Vec<f64>,
}


