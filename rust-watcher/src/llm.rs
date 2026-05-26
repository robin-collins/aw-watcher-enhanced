#![allow(dead_code)]
//! LLM integration for OCR text summarization.
//!
//! Supports multiple providers:
//! - Ollama (`/api/generate`)
//! - OpenAI-compatible chat APIs (`/chat/completions`)

use std::time::Duration;

use log::{debug, info};
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;

const TEXT_SUMMARIZE_PROMPT: &str = r#"Extract structured data from this screenshot OCR text.
- keywords: Up to 8 single-word keywords (no phrases)
- client: Client/company name or null
- project: Project name or null
- summary: One short sentence about the activity

OCR text:
"#;

/// JSON schema for structured output.
const JSON_SCHEMA: &str = r#"{"type":"object","properties":{"keywords":{"type":"array","items":{"type":"string"}},"client":{"type":["string","null"]},"project":{"type":["string","null"]},"summary":{"type":"string"}},"required":["keywords","summary"]}"#;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmSummary {
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub client: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: serde_json::Value,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Deserialize)]
struct OllamaModelListResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiAssistantMessage,
}

#[derive(Deserialize)]
struct OpenAiAssistantMessage {
    content: String,
}

#[derive(Deserialize)]
struct OpenAiModelListResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Deserialize)]
struct OpenAiModel {
    id: String,
}

pub struct LlmClient {
    client: Client,
    config: LlmConfig,
}

impl LlmClient {
    /// Build and validate a provider-specific LLM client.
    pub fn new(config: &LlmConfig) -> Result<Self, String> {
        config.validate()?;

        let client = Client::builder()
            .timeout(Duration::from_secs_f64(config.request_timeout))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    /// Run provider availability + model existence validation.
    pub fn validate_startup(&self) -> Result<(), String> {
        let models = self.list_models()?;
        if models.is_empty() {
            return Err(format!(
                "LLM provider '{}' at '{}' returned zero models",
                self.config.provider, self.config.base_url
            ));
        }

        if !models.iter().any(|m| m == &self.config.model) {
            return Err(format!(
                "Configured model '{}' not found in provider '{}'. Available models: {}",
                self.config.model,
                self.config.provider,
                models.join(", ")
            ));
        }

        info!(
            "LLM startup validation passed (provider={}, model={})",
            self.config.provider, self.config.model
        );
        Ok(())
    }

    pub fn summarize_ocr_with_context(
        &self,
        ocr_text: &str,
        app: &str,
        title: &str,
    ) -> Option<LlmSummary> {
        if !self.config.enabled || ocr_text.trim().is_empty() {
            return None;
        }

        let text = if ocr_text.len() > 2000 {
            let mut end = 2000;
            while !ocr_text.is_char_boundary(end) {
                end -= 1;
            }
            &ocr_text[..end]
        } else {
            ocr_text
        };

        let context = if !app.is_empty() {
            format!("\nActive app: {app}\nWindow title: {title}\n")
        } else {
            String::new()
        };
        let prompt = format!("{TEXT_SUMMARIZE_PROMPT}{context}\"{text}\"");

        let response = match self.config.provider.as_str() {
            "ollama" => self.send_ollama_request(&prompt),
            "openai_compatible" => self.send_openai_request(&prompt),
            other => Err(format!("Unsupported llm.provider '{other}'")),
        };

        match response {
            Ok(content) => parse_llm_response(&content),
            Err(e) => {
                debug!("LLM request failed: {e}");
                None
            }
        }
    }

    fn list_models(&self) -> Result<Vec<String>, String> {
        match self.config.provider.as_str() {
            "ollama" => {
                let url = format!("{}/api/tags", self.config.base_url.trim_end_matches('/'));
                let resp = self.send_with_auth(self.client.get(&url))?;
                let status = resp.status();
                let headers = resp.headers().clone();
                let text = resp
                    .text()
                    .map_err(|e| format!("Failed to read ollama model list response: {e}"))?;
                let resp_data = serde_json::from_str::<OllamaModelListResponse>(&text)
                    .map_err(|e| {
                        format!(
                            "Failed to parse ollama model list: {e}\n\
                            Request: GET {url}\n\
                            Response Status: {status}\n\
                            Response Headers: {headers:?}\n\
                            Response Body: {text}"
                        )
                    })?;
                Ok(resp_data.models.into_iter().map(|m| m.name).collect())
            }
            "openai_compatible" => {
                let url = format!("{}/models", self.config.base_url.trim_end_matches('/'));
                let resp = self.send_with_auth(self.client.get(&url))?;
                let status = resp.status();
                let headers = resp.headers().clone();
                let text = resp
                    .text()
                    .map_err(|e| format!("Failed to read OpenAI-compatible model list response: {e}"))?;
                let resp_data = serde_json::from_str::<OpenAiModelListResponse>(&text)
                    .map_err(|e| {
                        format!(
                            "Failed to parse OpenAI-compatible model list: {e}\n\
                            Request: GET {url}\n\
                            Response Status: {status}\n\
                            Response Headers: {headers:?}\n\
                            Response Body: {text}"
                        )
                    })?;
                Ok(resp_data.data.into_iter().map(|m| m.id).collect())
            }
            other => Err(format!("Unsupported llm.provider '{other}'")),
        }
    }

    fn send_ollama_request(&self, prompt: &str) -> Result<String, String> {
        let schema: serde_json::Value =
            serde_json::from_str(JSON_SCHEMA).map_err(|e| format!("Invalid schema: {e}"))?;
        let body = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            format: schema,
            options: OllamaOptions {
                temperature: 0.0,
                num_predict: 384,
            },
        };
        let url = format!(
            "{}/api/generate",
            self.config.base_url.trim_end_matches('/')
        );
        let resp = self.send_with_auth(self.client.post(&url).json(&body))?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = resp
            .text()
            .map_err(|e| format!("Failed to read ollama response: {e}"))?;
        let data = serde_json::from_str::<OllamaResponse>(&text)
            .map_err(|e| {
                format!(
                    "Failed to parse ollama response: {e}\n\
                    Request: POST {url}\n\
                    Request Body: {}\n\
                    Response Status: {status}\n\
                    Response Headers: {headers:?}\n\
                    Response Body: {text}",
                    serde_json::to_string(&body).unwrap_or_default()
                )
            })?;
        Ok(data.response)
    }

    fn send_openai_request(&self, prompt: &str) -> Result<String, String> {
        let schema_hint = "Return only raw JSON object with fields: keywords (array), client (string|null), project (string|null), summary (string).";
        let req = OpenAiChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".into(),
                    content: schema_hint.into(),
                },
                OpenAiMessage {
                    role: "user".into(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.0,
            max_tokens: 384,
        };
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let resp = self.send_with_auth(self.client.post(&url).json(&req))?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = resp
            .text()
            .map_err(|e| format!("Failed to read OpenAI-compatible response: {e}"))?;
        let data = serde_json::from_str::<OpenAiChatResponse>(&text)
            .map_err(|e| {
                format!(
                    "Failed to parse OpenAI-compatible response: {e}\n\
                    Request: POST {url}\n\
                    Request Body: {}\n\
                    Response Status: {status}\n\
                    Response Headers: {headers:?}\n\
                    Response Body: {text}",
                    serde_json::to_string(&req).unwrap_or_default()
                )
            })?;
        let content = data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "OpenAI-compatible response had no choices".to_string())?;
        Ok(content)
    }

    fn send_with_auth(&self, req: RequestBuilder) -> Result<reqwest::blocking::Response, String> {
        let req = if let Some(key) = self.config.api_key.as_deref() {
            req.bearer_auth(key)
        } else {
            req
        };

        let resp = req.send().map_err(|e| {
            if e.is_timeout() {
                format!("LLM request timeout after {}s", self.config.request_timeout)
            } else if e.is_connect() {
                format!("Failed to connect to '{}'", self.config.base_url)
            } else {
                format!("LLM network error: {e}")
            }
        })?;

        let status = resp.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err("Authentication failed: verify llm.api_key".into());
        }
        if !status.is_success() {
            let body_text = resp.text().unwrap_or_else(|_| "<failed to read response body>".to_string());
            return Err(format!(
                "LLM provider returned HTTP {} - Response Body: {}",
                status, body_text
            ));
        }
        Ok(resp)
    }
}

fn parse_llm_response(text: &str) -> Option<LlmSummary> {
    let trimmed = text.trim();
    if let Ok(summary) = serde_json::from_str::<LlmSummary>(trimmed) {
        return Some(summary);
    }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if start < end {
                let json_str = &trimmed[start..=end];
                if let Ok(summary) = serde_json::from_str::<LlmSummary>(json_str) {
                    return Some(summary);
                }
            }
        }
    }
    debug!(
        "Could not parse LLM response: {}",
        &trimmed[..trimmed.len().min(200)]
    );
    None
}
