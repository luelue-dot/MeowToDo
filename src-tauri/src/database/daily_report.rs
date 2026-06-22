use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use super::db::Database;
use super::models::DailyReport;
use crate::ai::service::AIService;
use crate::ai::settings::AiSettingsState;

/// Response from generating a daily report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportResult {
    pub report: DailyReport,
}

/// Query parameters for listing reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub keyword: Option<String>,
}

/// Paginated list of reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportListResult {
    pub reports: Vec<DailyReport>,
    pub total: i64,
}

/// Get today's task statistics for daily report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodayStats {
    created_count: i64,
    completed_count: i64,
    completion_rate: f64,
    completed_tasks: Vec<String>,
    pending_tasks: Vec<String>,
}

/// Generate daily report by analyzing today's tasks via AI
#[tauri::command]
pub fn generate_daily_report(
    db: State<Database>,
    ai_state: State<AiSettingsState>,
) -> Result<GenerateReportResult, String> {
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Check if a report already exists for today
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let existing: Result<String, _> = conn.query_row(
            "SELECT content FROM daily_reports WHERE report_date = ?1",
            rusqlite::params![&today],
            |row| row.get(0),
        );
        if let Ok(_) = existing {
            return Err("Today's report already exists.".to_string());
        }
    }

    // Gather today's task statistics
    let stats = get_today_stats(&db)?;

    // Build the AI prompt
    let prompt = format!(
        r#"你是一名效率教练。

请根据以下数据生成一份简洁日报。

数据：
- 今日创建任务数：{}
- 今日完成任务数：{}
- 完成率：{:.1}%
- 已完成任务列表：{}
- 未完成任务列表：{}

内容包括：
1. 今日完成情况
2. 今日亮点
3. 未完成事项
4. 明日建议

语气：积极、简洁

字数：200字以内"#,
        stats.created_count,
        stats.completed_count,
        stats.completion_rate,
        stats.completed_tasks.join(", "),
        stats.pending_tasks.join(", "),
    );

    // Call AI to generate the report
    let settings = ai_state.0.lock().map_err(|e| e.to_string())?;
    if settings.provider.is_empty() {
        return Err("AI provider is not configured. Please set up AI settings first.".to_string());
    }
    let service = AIService::new(settings.clone());
    let content = service.generate_text(&prompt)?;

    // Save the report to database
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO daily_reports (report_date, content, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![&today, &content, &now],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let report = DailyReport {
        id,
        report_date: today,
        content: content.clone(),
        created_at: Some(now),
    };

    Ok(GenerateReportResult { report })
}

/// Generate daily report silently (for scheduled tasks)
#[allow(unused)]
pub fn generate_daily_report_silent(
    app_handle: &AppHandle,
) -> Result<DailyReport, String> {
    let db = app_handle.state::<Database>();
    let ai_state = app_handle.state::<AiSettingsState>();
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Skip if already exists for today
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let existing: Result<String, _> = conn.query_row(
            "SELECT content FROM daily_reports WHERE report_date = ?1",
            rusqlite::params![&today],
            |row| row.get(0),
        );
        if let Ok(_) = existing {
            return Err("Today's report already exists.".to_string());
        }
    }

    let stats = get_today_stats_internal(&db)?;

    let prompt = format!(
        r#"你是一名效率教练。

请根据以下数据生成一份简洁日报。

数据：
- 今日创建任务数：{}
- 今日完成任务数：{}
- 完成率：{:.1}%
- 已完成任务列表：{}
- 未完成任务列表：{}

内容包括：
1. 今日完成情况
2. 今日亮点
3. 未完成事项
4. 明日建议

语气：积极、简洁

字数：200字以内"#,
        stats.created_count,
        stats.completed_count,
        stats.completion_rate,
        stats.completed_tasks.join(", "),
        stats.pending_tasks.join(", "),
    );

    let settings = ai_state.0.lock().map_err(|e| e.to_string())?;
    let service = AIService::new(settings.clone());
    drop(settings);

    let content = service.generate_text(&prompt)?;

    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO daily_reports (report_date, content, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![&today, &content, &now],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let report = DailyReport {
        id,
        report_date: today,
        content,
        created_at: Some(now),
    };

    // Emit event to notify frontend
    let _ = app_handle.emit("daily-report-generated", &report);

    Ok(report)
}

/// Get daily reports with pagination, date range filter, and keyword search
#[tauri::command]
pub fn get_daily_reports(
    db: State<Database>,
    query: ReportQuery,
) -> Result<ReportListResult, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1).min(100);
    let offset = (page - 1) * page_size;

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref start) = query.start_date {
        params.push(Box::new(start.clone()));
        conditions.push(format!("report_date >= ?{}", params.len()));
    }
    if let Some(ref end) = query.end_date {
        params.push(Box::new(end.clone()));
        conditions.push(format!("report_date <= ?{}", params.len()));
    }
    if let Some(ref kw) = query.keyword {
        if !kw.is_empty() {
            params.push(Box::new(format!("%{}%", kw)));
            conditions.push(format!("content LIKE ?{}", params.len()));
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Get total count
    let count_sql = format!("SELECT COUNT(*) FROM daily_reports {}", where_clause);
    let total: i64 = {
        let mut stmt = conn.prepare(&count_sql).map_err(|e| e.to_string())?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        stmt.query_row(param_refs.as_slice(), |row| row.get(0))
            .map_err(|e| e.to_string())?
    };

    // Get paginated results
    params.push(Box::new(page_size));
    params.push(Box::new(offset));
    let data_sql = format!(
        "SELECT id, report_date, content, created_at FROM daily_reports {} ORDER BY report_date DESC LIMIT ?{} OFFSET ?{}",
        where_clause,
        params.len() - 1,
        params.len()
    );

    let mut stmt = conn.prepare(&data_sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let reports = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(DailyReport {
                id: row.get(0)?,
                report_date: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(ReportListResult { reports, total })
}

/// Get today's task stats (for command usage)
fn get_today_stats(db: &State<Database>) -> Result<TodayStats, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_stats_from_conn(&conn)
}

#[allow(unused)]
fn get_today_stats_internal(db: &Database) -> Result<TodayStats, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_stats_from_conn(&conn)
}

fn get_stats_from_conn(conn: &rusqlite::Connection) -> Result<TodayStats, String> {
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Total created today
    let created_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM todos WHERE task_date = ?1",
            rusqlite::params![&today],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Completed today
    let completed_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM todos WHERE task_date = ?1 AND status = 'completed'",
            rusqlite::params![&today],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Completion rate
    let completion_rate = if created_count > 0 {
        (completed_count as f64 / created_count as f64) * 100.0
    } else {
        0.0
    };

    // Completed task titles
    let mut stmt = conn
        .prepare(
            "SELECT title FROM todos WHERE task_date = ?1 AND status = 'completed' ORDER BY completed_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let completed_tasks: Vec<String> = stmt
        .query_map(rusqlite::params![&today], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Pending (non-completed) task titles
    let mut stmt = conn
        .prepare(
            "SELECT title FROM todos WHERE task_date = ?1 AND (status IS NULL OR status != 'completed') ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let pending_tasks: Vec<String> = stmt
        .query_map(rusqlite::params![&today], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(TodayStats {
        created_count,
        completed_count,
        completion_rate,
        completed_tasks,
        pending_tasks,
    })
}
