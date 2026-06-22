use serde::{Deserialize, Serialize};

use super::service::{AIService, extract_json_from_markdown};
use super::settings::AiSettingsState;

/// Response from the decompose_task command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposeResult {
    pub subtasks: Vec<SubtaskItem>,
}

/// A single subtask with title and priority from AI decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskItem {
    pub title: String,
    pub priority: i64,
}

/// Request for the decompose_task command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposeRequest {
    pub task_title: String,
}

#[tauri::command]
pub fn decompose_task(
    state: tauri::State<AiSettingsState>,
    task: DecomposeRequest,
) -> Result<DecomposeResult, String> {
    let settings = state.0.lock().map_err(|e| e.to_string())?;

    if settings.provider.is_empty() {
        return Err("AI provider is not configured. Please set up AI settings first.".to_string());
    }

    let prompt = format!(
        r#"你是一名任务规划专家。

请将用户输入的大型目标拆解为5-10个具体、可执行、可验证的任务。

要求：
1. 每个任务足够细化
2. 每个任务能在2小时内完成
3. 为每个任务分配优先级（1=高优先级，2=中优先级，0=低优先级）
4. 只输出JSON数组，不要其他内容，格式为：[{{"title": "任务标题", "priority": 1}}, ...]

输入：{}

输出："#,
        task.task_title
    );

    let service = AIService::new(settings.clone());
    let response = service.generate_text(&prompt)?;

    let cleaned = extract_json_from_markdown(&response);

    let subtasks: Vec<SubtaskItem> = serde_json::from_str(&cleaned).map_err(|e| {
        format!(
            "Failed to parse subtasks from AI response: {}\nRaw response: {}",
            e, response
        )
    })?;

    Ok(DecomposeResult { subtasks })
}
