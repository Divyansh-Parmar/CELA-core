use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::EngineError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOptions {
    pub max_tokens: Option<u32>,
    pub max_time_ms: Option<u64>,
    pub temperature: Option<f32>,
    pub stop_sequences: Vec<String>,
}

impl Default for InferenceOptions {
    fn default() -> Self {
        Self {
            max_tokens: Some(128),
            max_time_ms: Some(30000), // 30s default timeout
            temperature: Some(0.0),
            stop_sequences: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLoadConfig {
    pub model_path: PathBuf,
    pub context_size: usize,
    pub gpu_layers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub text: String,
    pub usage: Usage,
    pub status: InferenceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InferenceStatus {
    Success,
    Truncated,
    Error,
}

#[async_trait]
pub trait ModelRuntime: Send + Sync {
    /// Initialize and load the model.
    async fn load(&mut self, config: &ModelLoadConfig) -> Result<(), EngineError>;

    /// Perform inference with strict limits.
    async fn infer(&mut self, prompt: &str, options: InferenceOptions) -> Result<InferenceResult, EngineError>;

    /// Unload the model to free resources.
    async fn unload(&mut self) -> Result<(), EngineError>;
}