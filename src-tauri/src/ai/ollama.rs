use std::time::Duration;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::ai::prompts::VISION_SYSTEM_PROMPT;
use crate::ai::provider::{
    EmbeddingProvider, ModelInfo, RuntimeProvider, VisionAnalysisResponse, VisionProvider,
};
use crate::core::errors::{AppError, Result};

pub struct OllamaClient {
    base_url: String,
    client: Client,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }
}

impl RuntimeProvider for OllamaClient {
    fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/version", self.base_url);
        let res = self.client.get(&url).send();
        match res {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModelTag>>,
}

#[derive(Deserialize)]
struct OllamaModelTag {
    name: String,
    size: Option<u64>,
    modified_at: Option<String>,
}

impl OllamaClient {
    pub fn fetch_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| AppError::Ollama(format!("Failed to connect to Ollama: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::Ollama(format!(
                "Ollama tags request returned HTTP {}",
                resp.status()
            )));
        }

        let parsed: OllamaTagsResponse = resp
            .json()
            .map_err(|e| AppError::Ollama(format!("Failed to parse Ollama models response: {}", e)))?;

        let mut models = Vec::new();
        if let Some(list) = parsed.models {
            for m in list {
                let name_lower = m.name.to_lowercase();
                let is_embedding = name_lower.contains("embed")
                    || name_lower.contains("bge")
                    || name_lower.contains("nomic");
                let is_vision = name_lower.contains("vision")
                    || name_lower.contains("llava")
                    || name_lower.contains("qwen")
                    || name_lower.contains("ministral")
                    || name_lower.contains("gemma")
                    || name_lower.contains("fable")
                    || name_lower.contains("granite");

                models.push(ModelInfo {
                    name: m.name,
                    size_bytes: m.size.unwrap_or(0),
                    modified_at: m.modified_at.unwrap_or_default(),
                    is_vision,
                    is_embedding,
                });
            }
        }

        Ok(models)
    }
}

impl VisionProvider for OllamaClient {
    fn check_availability(&self) -> Result<bool> {
        self.health_check()
    }

    fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let all = self.fetch_models()?;
        Ok(all.into_iter().filter(|m| m.is_vision).collect())
    }

    fn analyze_image(&self, model: &str, image_base64: &str) -> Result<VisionAnalysisResponse> {
        let url = format!("{}/api/generate", self.base_url);

        let body = json!({
            "model": model,
            "prompt": "Analyze this image and describe its subject, aesthetic style, category, lighting, colors, and environment in the requested JSON structure.",
            "system": VISION_SYSTEM_PROMPT,
            "images": [image_base64],
            "stream": false,
            "format": "json",
            "options": {
                "num_predict": 180,
                "temperature": 0.1
            }
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| AppError::Ollama(format!("Ollama generate call failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().unwrap_or_default();
            return Err(AppError::Ollama(format!(
                "Ollama responded with HTTP {}: {}",
                status, err_text
            )));
        }

        let json_resp: Value = resp
            .json()
            .map_err(|e| AppError::Ollama(format!("Failed to parse response: {}", e)))?;

        let response_text = json_resp
            .get("response")
            .and_then(|r| r.as_str())
            .ok_or_else(|| AppError::Ollama("Response field missing from Ollama output".to_string()))?;

        // Parse structured JSON output
        let parsed: Value = serde_json::from_str(response_text)
            .or_else(|_| {
                // If wrapped in markdown code fence, strip it
                let cleaned = response_text
                    .trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();
                serde_json::from_str(cleaned)
            })
            .map_err(|e| {
                AppError::Validation(format!(
                    "Model output could not be parsed as valid JSON: {}. Raw: {}",
                    e, response_text
                ))
            })?;

        let description = parsed
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Visual image")
            .to_string();

        let subject = parse_string_array(parsed.get("subject"));
        let category = parsed
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("Wallpapers")
            .to_string();

        let subcategory = parsed
            .get("subcategory")
            .and_then(|v| v.as_str())
            .unwrap_or("General")
            .to_string();

        let style = parse_string_array(parsed.get("style"));
        let environment = parsed
            .get("environment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let lighting = parsed
            .get("lighting")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let colors = parse_string_array(parsed.get("colors"));
        let orientation = parsed
            .get("orientation")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let suggested_filename = parsed
            .get("suggested_filename")
            .and_then(|v| v.as_str())
            .unwrap_or("image.png")
            .to_string();

        Ok(VisionAnalysisResponse {
            description,
            subject,
            category,
            subcategory,
            style,
            environment,
            lighting,
            colors,
            orientation,
            suggested_filename,
        })
    }
}

fn parse_string_array(val: Option<&Value>) -> Vec<String> {
    match val {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|item| item.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Some(Value::String(s)) => s
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

impl EmbeddingProvider for OllamaClient {
    fn check_availability(&self) -> Result<bool> {
        self.health_check()
    }

    fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let all = self.fetch_models()?;
        Ok(all.into_iter().filter(|m| m.is_embedding).collect())
    }

    fn embed_text(&self, model: &str, text: &str) -> Result<Vec<f32>> {
        // Try modern /api/embed first
        let embed_url = format!("{}/api/embed", self.base_url);
        let body = json!({
            "model": model,
            "input": text
        });

        if let Ok(resp) = self.client.post(&embed_url).json(&body).send() {
            if resp.status().is_success() {
                if let Ok(val) = resp.json::<Value>() {
                    if let Some(embeddings) = val.get("embeddings").and_then(|e: &Value| e.as_array()) {
                        if let Some(first) = embeddings.first().and_then(|f: &Value| f.as_array()) {
                            let vec: Vec<f32> = first
                                .iter()
                                .filter_map(|v: &Value| v.as_f64().map(|f| f as f32))
                                .collect();
                            if !vec.is_empty() {
                                return Ok(vec);
                            }
                        }
                    }
                }
            }
        }

        // Fallback to legacy /api/embeddings
        let legacy_url = format!("{}/api/embeddings", self.base_url);
        let legacy_body = json!({
            "model": model,
            "prompt": text
        });

        let resp = self
            .client
            .post(&legacy_url)
            .json(&legacy_body)
            .send()
            .map_err(|e| AppError::Ollama(format!("Ollama embedding request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::Ollama(format!(
                "Ollama embeddings returned HTTP {}",
                resp.status()
            )));
        }

        let val: Value = resp
            .json()
            .map_err(|e| AppError::Ollama(format!("Failed to parse embedding response: {}", e)))?;

        let embedding_arr = val
            .get("embedding")
            .and_then(|e: &Value| e.as_array())
            .ok_or_else(|| AppError::Ollama("Missing embedding array in response".to_string()))?;

        let vec: Vec<f32> = embedding_arr
            .iter()
            .filter_map(|v: &Value| v.as_f64().map(|f| f as f32))
            .collect();

        if vec.is_empty() {
            return Err(AppError::Ollama("Empty embedding vector received".to_string()));
        }

        Ok(vec)
    }

    fn embed_batch(&self, model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for t in texts {
            results.push(self.embed_text(model, t)?);
        }
        Ok(results)
    }
}
