use serde::{Deserialize, Serialize};


use super::settings::AiSettings;

/// Unified request for LLM text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTextRequest {
    pub prompt: String,
}

/// Unified response from LLM text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTextResponse {
    pub content: String,
}

/// Provider types supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Openai,
    Deepseek,
    Ollama,
}

impl ProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderType::Openai => "openai",
            ProviderType::Deepseek => "deepseek",
            ProviderType::Ollama => "ollama",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(ProviderType::Openai),
            "deepseek" => Some(ProviderType::Deepseek),
            "ollama" => Some(ProviderType::Ollama),
            _ => None,
        }
    }
}

/// Core AI Service that dispatches to the right provider
pub struct AIService {
    pub settings: AiSettings,
}

impl AIService {
    pub fn new(settings: AiSettings) -> Self {
        Self { settings }
    }

    /// Unified text generation — the single entry point for all AI features
    pub fn generate_text(&self, prompt: &str) -> Result<String, String> {
        let provider = ProviderType::from_str(&self.settings.provider)
            .ok_or_else(|| format!("Unsupported provider: {}", self.settings.provider))?;

        match provider {
            ProviderType::Openai => self.invoke_openai(prompt),
            ProviderType::Deepseek => self.invoke_openai_compat(prompt, &self.settings.base_url),
            ProviderType::Ollama => self.invoke_ollama(prompt),
        }
    }

    /// OpenAI-compatible API (also used by DeepSeek with different base_url)
    fn invoke_openai(&self, prompt: &str) -> Result<String, String> {
        let base_url = if self.settings.base_url.is_empty() {
            "https://api.openai.com/v1".to_string()
        } else {
            self.settings.base_url.clone()
        };
        self.invoke_openai_compat(prompt, &base_url)
    }

    fn invoke_openai_compat(&self, prompt: &str, base_url: &str) -> Result<String, String> {
        let api_key = &self.settings.api_key;
        if api_key.is_empty() {
            return Err("API Key is not configured".to_string());
        }

        let body = serde_json::json!({
            "model": self.settings.model_name,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "stream": false
        });

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let resp = client
            .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Failed to call LLM API: {}", e))?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

        if !status.is_success() {
            let err_msg = json["error"]["message"].as_str().unwrap_or("unknown error");
            return Err(format!("API error ({}): {}", status, err_msg));
        }

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("Missing 'content' in LLM response")?
            .to_string();

        Ok(content)
    }

    fn invoke_ollama(&self, prompt: &str) -> Result<String, String> {
        let base_url = if self.settings.base_url.is_empty() {
            "http://localhost:11434".to_string()
        } else {
            self.settings.base_url.clone()
        };

        let body = serde_json::json!({
            "model": self.settings.model_name,
            "stream": false,
            "prompt": prompt
        });

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let resp = client
            .post(format!("{}/api/generate", base_url.trim_end_matches('/')))
            .json(&body)
            .send()
            .map_err(|e| format!("Failed to call Ollama API: {}", e))?;

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let text = json["response"]
            .as_str()
            .ok_or("Missing 'response' field in Ollama output")?;

        Ok(text.to_string())
    }
}

/// Convert a JSON value that might contain a markdown JSON block into a parsable string
pub fn extract_json_from_markdown(text: &str) -> String {
    text.trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}

/// Tauri command: invoke LLM to generate text
#[tauri::command]
pub fn invoke_llm(
    db: tauri::State<super::settings::AiSettingsState>,
    request: GenerateTextRequest,
) -> Result<GenerateTextResponse, String> {
    let settings = db.0.lock().map_err(|e| e.to_string())?;

    if settings.provider.is_empty() {
        return Err("AI provider is not configured. Please set up AI settings first.".to_string());
    }

    let service = AIService::new(settings.clone());
    let content = service.generate_text(&request.prompt)?;
    Ok(GenerateTextResponse { content })
}
