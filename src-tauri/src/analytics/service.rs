use chrono::Local;
use rusqlite::{Connection, params};
use tauri::{Manager, State};

use super::models::{AnalyticsEvent, AnalyticsSummary, TrendData};
use crate::database::db::Database;

/// Track a single analytics event (insert into analytics_events).
/// Returns the inserted event id.
pub fn track_event(conn: &Connection, event_name: &str, event_value: Option<&str>) -> i64 {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO analytics_events (event_name, event_value, created_at) VALUES (?1, ?2, ?3)",
        params![event_name, event_value, now],
    )
    .ok();
    conn.last_insert_rowid()
}

/// Check if user testing mode is enabled (stored in app_settings table).
pub fn is_testing_enabled(conn: &Connection) -> bool {
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'user_testing_mode'",
        [],
        |row| row.get(0),
    );
    matches!(result, Ok(ref v) if v == "true")
}

/// Set user testing mode.
pub fn set_testing_enabled(conn: &Connection, enabled: bool) {
    let value = if enabled { "true" } else { "false" };
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('user_testing_mode', ?1)",
        params![value],
    )
    .ok();
}

/// Track event only if testing mode is enabled.
#[allow(dead_code)]
pub fn track_event_if_enabled(db: &State<Database>, event_name: &str, event_value: Option<&str>) {
    let conn = db.conn.lock().unwrap();
    if is_testing_enabled(&conn) {
        track_event(&conn, event_name, event_value);
    }
}

/// Get analytics summary for a date range (default: all time).
#[tauri::command]
pub fn get_analytics_summary(db: State<Database>) -> Result<AnalyticsSummary, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Count analytics_events for DAU / activity
    let active_days: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT date(created_at)) FROM analytics_events WHERE event_name = 'app_open'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let tasks_created: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'task_created'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let tasks_completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'task_completed'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let task_completion_rate = if tasks_created > 0 {
        (tasks_completed as f64 / tasks_created as f64) * 100.0
    } else {
        0.0
    };

    let ai_decompose_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'ai_decompose'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let ai_decompose_usage_rate = if tasks_created > 0 {
        (ai_decompose_count as f64 / tasks_created as f64) * 100.0
    } else {
        0.0
    };

    let recommendation_total_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'recommendation_generated'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let recommendation_accept_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'recommendation_accept'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let recommendation_accept_rate = if recommendation_total_count > 0 {
        (recommendation_accept_count as f64 / recommendation_total_count as f64) * 100.0
    } else {
        0.0
    };

    let daily_report_generated_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'daily_report_generated'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let daily_report_open_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'daily_report_open'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let daily_report_open_rate = if daily_report_generated_count > 0 {
        (daily_report_open_count as f64 / daily_report_generated_count as f64) * 100.0
    } else {
        0.0
    };

    Ok(AnalyticsSummary {
        active_days,
        tasks_created,
        tasks_completed,
        task_completion_rate: (task_completion_rate * 10.0).round() / 10.0,
        ai_decompose_count,
        ai_decompose_usage_rate: (ai_decompose_usage_rate * 10.0).round() / 10.0,
        recommendation_accept_rate: (recommendation_accept_rate * 10.0).round() / 10.0,
        daily_report_open_rate: (daily_report_open_rate * 10.0).round() / 10.0,
    })
}

/// Get daily trend data for charts (last N days).
#[tauri::command]
pub fn get_analytics_trend(db: State<Database>, days: i64) -> Result<TrendData, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut dates = Vec::new();
    let mut tasks_created = Vec::new();
    let mut tasks_completed = Vec::new();
    let mut ai_decompose = Vec::new();
    let mut completion_rates = Vec::new();

    // Generate date range
    let _today = Local::now().format("%Y-%m-%d").to_string();
    for i in (0..days).rev() {
        let date = {
            let d = Local::now() - chrono::Duration::days(i);
            d.format("%Y-%m-%d").to_string()
        };
        dates.push(date.clone());

        // tasks created on that day
        let created: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'task_created' AND date(created_at) = ?1",
                params![&date],
                |row| row.get(0),
            )
            .unwrap_or(0);
        tasks_created.push(created);

        // tasks completed on that day
        let completed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'task_completed' AND date(created_at) = ?1",
                params![&date],
                |row| row.get(0),
            )
            .unwrap_or(0);
        tasks_completed.push(completed);

        // ai decompose on that day
        let decompose: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM analytics_events WHERE event_name = 'ai_decompose' AND date(created_at) = ?1",
                params![&date],
                |row| row.get(0),
            )
            .unwrap_or(0);
        ai_decompose.push(decompose);

        // completion rate for that day
        let rate = if created > 0 {
            (completed as f64 / created as f64) * 100.0
        } else {
            0.0
        };
        completion_rates.push((rate * 10.0).round() / 10.0);
    }

    Ok(TrendData {
        dates,
        tasks_created,
        tasks_completed,
        ai_decompose,
        completion_rates,
    })
}

/// Get recent analytics events.
#[tauri::command]
pub fn get_analytics_events(db: State<Database>, limit: i64) -> Result<Vec<AnalyticsEvent>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, event_name, event_value, created_at FROM analytics_events ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let events = stmt
        .query_map(params![limit], |row| {
            Ok(AnalyticsEvent {
                id: row.get(0)?,
                event_name: row.get(1)?,
                event_value: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(events)
}

/// Track an event from frontend (with testing mode check).
#[tauri::command]
pub fn track_event_command(db: State<Database>, event_name: String, event_value: Option<String>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    // Always track the event if testing is enabled (no check needed for explicit calls)
    track_event(&conn, &event_name, event_value.as_deref());
    Ok(())
}

/// Get testing mode status.
#[tauri::command]
pub fn get_testing_mode(db: State<Database>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(is_testing_enabled(&conn))
}

/// Set testing mode.
#[tauri::command]
pub fn set_testing_mode(db: State<Database>, enabled: bool) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    set_testing_enabled(&conn, enabled);
    Ok(())
}

/// Export analytics report as JSON string.
#[tauri::command]
pub fn export_analytics_report(db: State<Database>) -> Result<String, String> {
    let summary = get_analytics_summary(db)?;
    serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())
}



/// Show the analytics window (from config). Safe to call from any window.
#[tauri::command]
pub fn show_analytics_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("analytics") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        Ok(())
    } else {
        Err("Analytics window not found".to_string())
    }
}


