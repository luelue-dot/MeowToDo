use chrono::Local;
use tauri::{AppHandle, Emitter, Manager, State};

use super::db::Database;
use super::models::{ActivityLog, CreateTodoRequest, Todo, UpdateTodoRequest};

fn log_activity(conn: &rusqlite::Connection, action: &str, task_id: Option<i64>) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO activity_logs (action, task_id, timestamp) VALUES (?1, ?2, ?3)",
        rusqlite::params![action, task_id, now],
    )
    .ok();
}

#[tauri::command]
pub fn create_task(db: State<Database>, task: CreateTodoRequest) -> Result<Todo, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let today = Local::now().format("%Y-%m-%d").to_string();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let status = task.status.unwrap_or_else(|| String::from("pending"));

    conn.execute(
        "INSERT INTO todos (title, description, status, priority, deadline, created_at, task_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![task.title, task.description, status, task.priority, task.deadline, now, task.task_date.as_deref().unwrap_or(&today)],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    log_activity(&conn, "create", Some(id));

    let todo = conn
        .query_row(
            "SELECT id, title, description, status, priority, deadline, created_at, completed_at, task_date FROM todos WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    priority: row.get(4)?,
                    deadline: row.get(5)?,
                    created_at: row.get(6)?,
                    completed_at: row.get(7)?,
                    task_date: row.get(8)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(todo)
}

#[tauri::command]
pub fn update_task(db: State<Database>, task: UpdateTodoRequest) -> Result<Todo, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let existing = conn
        .query_row(
            "SELECT title, description, status, priority, deadline FROM todos WHERE id = ?1",
            rusqlite::params![task.id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let title = task.title.unwrap_or(existing.0);
    let description = task.description.or(existing.1);
    let status = task.status.or(existing.2);
    let priority = task.priority.or(existing.3);
    let deadline = task.deadline.or(existing.4);

    conn.execute(
        "UPDATE todos SET title = ?1, description = ?2, status = ?3, priority = ?4, deadline = ?5 WHERE id = ?6",
        rusqlite::params![title, description, status, priority, deadline, task.id],
    )
    .map_err(|e| e.to_string())?;

    log_activity(&conn, "update", Some(task.id));

    let todo = conn
        .query_row(
            "SELECT id, title, description, status, priority, deadline, created_at, completed_at, task_date FROM todos WHERE id = ?1",
            rusqlite::params![task.id],
            |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    priority: row.get(4)?,
                    deadline: row.get(5)?,
                    created_at: row.get(6)?,
                    completed_at: row.get(7)?,
                    task_date: row.get(8)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(todo)
}

#[tauri::command]
pub fn delete_task(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    log_activity(&conn, "delete", Some(id));

    conn.execute("DELETE FROM todos WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn complete_task(app: AppHandle, db: State<Database>, id: i64) -> Result<Todo, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "UPDATE todos SET status = 'completed', completed_at = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    )
    .map_err(|e| e.to_string())?;

    log_activity(&conn, "complete", Some(id));

    let todo = conn
        .query_row(
            "SELECT id, title, description, status, priority, deadline, created_at, completed_at, task_date FROM todos WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    priority: row.get(4)?,
                    deadline: row.get(5)?,
                    created_at: row.get(6)?,
                    completed_at: row.get(7)?,
                    task_date: row.get(8)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    // Emit task-completed event to the main window so the cat can react
    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.emit("task-completed", &todo);
    }

    Ok(todo)
}

#[tauri::command]
pub fn get_today_tasks(db: State<Database>) -> Result<Vec<Todo>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, status, priority, deadline, created_at, completed_at, task_date 
             FROM todos 
             WHERE task_date = ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let todos = stmt
        .query_map(rusqlite::params![today], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                priority: row.get(4)?,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
                completed_at: row.get(7)?,
                task_date: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(todos)
}

#[tauri::command]
pub fn get_task_history(db: State<Database>) -> Result<Vec<Todo>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, status, priority, deadline, created_at, completed_at, task_date 
             FROM todos 
             ORDER BY created_at DESC
             LIMIT 200",
        )
        .map_err(|e| e.to_string())?;

    let todos = stmt
        .query_map([], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                priority: row.get(4)?,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
                completed_at: row.get(7)?,
                task_date: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(todos)
}

#[tauri::command]
pub fn get_activity_logs(db: State<Database>) -> Result<Vec<ActivityLog>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, action, task_id, timestamp 
             FROM activity_logs 
             ORDER BY timestamp DESC 
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let logs = stmt
        .query_map([], |row| {
            Ok(ActivityLog {
                id: row.get(0)?,
                action: row.get(1)?,
                task_id: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(logs)
}








