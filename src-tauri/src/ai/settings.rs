use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::database::db::Database;

/// AI settings model matching the ai_settings table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub id: i64,
    pub provider: String,
    pub model_name: String,
    pub api_key: String,
    pub base_url: String,
    pub is_active: bool,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            id: 0,
            provider: "ollama".to_string(),
            model_name: "qwen2.5:7b".to_string(),
            api_key: String::new(),
            base_url: String::new(),
            is_active: true,
        }
    }
}

/// Request to save/update AI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAiSettingsRequest {
    pub provider: String,
    pub model_name: String,
    pub api_key: String,
    pub base_url: String,
}

/// Managed state for active AI settings
pub struct AiSettingsState(pub Mutex<AiSettings>);

/// Get current active AI settings from DB
#[tauri::command]
pub fn get_ai_settings(db: tauri::State<Database>) -> Result<AiSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT id, provider, model_name, api_key, base_url, is_active FROM ai_settings WHERE is_active = 1 LIMIT 1",
        [],
        |row| {
            Ok(AiSettings {
                id: row.get(0)?,
                provider: row.get(1)?,
                model_name: row.get(2)?,
                api_key: row.get(3)?,
                base_url: row.get(4)?,
                is_active: row.get(5)?,
            })
        },
    );

    match result {
        Ok(settings) => Ok(settings),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AiSettings::default()),
        Err(e) => Err(e.to_string()),
    }
}

/// Save or update AI settings
#[tauri::command]
pub fn save_ai_settings(
    db: tauri::State<Database>,
    state: tauri::State<AiSettingsState>,
    request: SaveAiSettingsRequest,
) -> Result<AiSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if any active setting exists
    let existing: Result<i64, _> = conn.query_row(
        "SELECT id FROM ai_settings WHERE is_active = 1 LIMIT 1",
        [],
        |row| row.get(0),
    );

    match existing {
        Ok(id) => {
            // Update existing
            conn.execute(
                "UPDATE ai_settings SET provider = ?1, model_name = ?2, api_key = ?3, base_url = ?4 WHERE id = ?5",
                rusqlite::params![request.provider, request.model_name, request.api_key, request.base_url, id],
            )
            .map_err(|e| e.to_string())?;

            let settings = AiSettings {
                id,
                provider: request.provider,
                model_name: request.model_name,
                api_key: request.api_key,
                base_url: request.base_url,
                is_active: true,
            };

            // Update managed state
            if let Ok(mut state) = state.0.lock() {
                *state = settings.clone();
            }

            Ok(settings)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Insert new
            conn.execute(
                "INSERT INTO ai_settings (provider, model_name, api_key, base_url, is_active) VALUES (?1, ?2, ?3, ?4, 1)",
                rusqlite::params![request.provider, request.model_name, request.api_key, request.base_url],
            )
            .map_err(|e| e.to_string())?;

            let id = conn.last_insert_rowid();

            let settings = AiSettings {
                id,
                provider: request.provider,
                model_name: request.model_name,
                api_key: request.api_key,
                base_url: request.base_url,
                is_active: true,
            };

            // Update managed state
            if let Ok(mut state) = state.0.lock() {
                *state = settings.clone();
            }

            Ok(settings)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Test the current AI connection
#[tauri::command]
pub fn test_ai_connection(
    db: tauri::State<Database>,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let settings: AiSettings = conn.query_row(
        "SELECT id, provider, model_name, api_key, base_url, is_active FROM ai_settings WHERE is_active = 1 LIMIT 1",
        [],
        |row| {
            Ok(AiSettings {
                id: row.get(0)?,
                provider: row.get(1)?,
                model_name: row.get(2)?,
                api_key: row.get(3)?,
                base_url: row.get(4)?,
                is_active: row.get(5)?,
            })
        },
    )
    .map_err(|e| format!("No AI configuration found: {}", e))?;

    let service = super::service::AIService::new(settings);
    let content = service.generate_text("你好，请用一句话介绍自己。")?;
    Ok(content)
}
