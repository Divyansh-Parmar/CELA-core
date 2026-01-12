pub mod config;
pub mod error;
pub mod runtime;
pub mod memory;

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::EngineConfig;
use crate::error::EngineError;
use crate::runtime::{ModelRuntime, ModelLoadConfig, InferenceOptions, InferenceResult, InferenceStatus, Usage};
use crate::memory::MemoryManager;
use serde::{Deserialize, Serialize};

/// The main entry point for the Local AI Engine.
pub struct Engine {
    config: EngineConfig,
    runtime: Arc<Mutex<Box<dyn ModelRuntime>>>,
    pub memory: Arc<MemoryManager>,
}

/// The standard JSON output for all engine requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResponse {
    pub status: String,
    pub intent: Option<String>,
    pub output: OutputContent,
    pub usage: Usage,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputContent {
    pub text: String,
}

impl Engine {
    pub fn new(config: EngineConfig, runtime: Box<dyn ModelRuntime>) -> Self {
        let memory_config = config.memory.clone();
        Self {
            config,
            runtime: Arc::new(Mutex::new(runtime)),
            memory: Arc::new(MemoryManager::new(memory_config)),
        }
    }

    pub async fn init(&self) -> Result<(), EngineError> {
        let mut runtime = self.runtime.lock().await;
        
        let load_config = ModelLoadConfig {
            model_path: self.config.model.default_path.clone(),
            context_size: self.config.model.default_context_size,
            gpu_layers: self.config.model.default_gpu_layers,
        };

        runtime.load(&load_config).await?;
        Ok(())
    }

    pub async fn process_request(&self, prompt: &str, options: InferenceOptions) -> Result<EngineResponse, EngineError> {
        // 1. Get Memory Injection
        let memory_context = self.memory.get_injection_text().await;
        
        // 2. Construct final prompt
        let final_prompt = if !memory_context.is_empty() {
            format!("{}{}", memory_context, prompt)
        } else {
            prompt.to_string()
        };
        
        // 3. Inference
        let mut runtime = self.runtime.lock().await;
        let result = runtime.infer(&final_prompt, options).await;

        match result {
            Ok(inf_result) => {
                let status_str = match inf_result.status {
                    InferenceStatus::Success => "success",
                    InferenceStatus::Truncated => "truncated",
                    InferenceStatus::Error => "error",
                }.to_string();

                Ok(EngineResponse {
                    status: status_str,
                    intent: None,
                    output: OutputContent {
                        text: inf_result.text,
                    },
                    usage: inf_result.usage,
                    error: None,
                })
            }
            Err(e) => {
                Ok(EngineResponse {
                    status: "error".to_string(),
                    intent: None,
                    output: OutputContent { text: "".to_string() },
                    usage: Usage::default(),
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockRuntime;

    #[async_trait]
    impl ModelRuntime for MockRuntime {
        async fn load(&mut self, _config: &ModelLoadConfig) -> Result<(), EngineError> {
            Ok(())
        }

        async fn infer(&mut self, prompt: &str, _options: InferenceOptions) -> Result<InferenceResult, EngineError> {
            Ok(InferenceResult {
                text: format!("Mock response to: {}", prompt),
                usage: Usage {
                    input_tokens: 5,
                    output_tokens: 10,
                    total_tokens: 15,
                    duration_ms: 10,
                },
                status: InferenceStatus::Success,
            })
        }

        async fn unload(&mut self) -> Result<(), EngineError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_engine_flow() {
        let config = EngineConfig::default();
        let runtime = MockRuntime;
        let engine = Engine::new(config, Box::new(runtime));

        let response = engine.process_request("Hello", InferenceOptions::default()).await.unwrap();
        assert_eq!(response.status, "success");
        // Verify prompt pass-through
        assert_eq!(response.output.text, "Mock response to: Hello");
    }

    #[tokio::test]
    async fn test_memory_injection() {
        let mut config = EngineConfig::default();
        config.memory.enabled = true;
        // memory.json path? Use a temp file or defaults (but default path might not be writable in some CI)
        // For test, we trust default logic but we can manually set fact.
        
        let runtime = MockRuntime;
        let engine = Engine::new(config, Box::new(runtime));
        
        // Inject fact
        engine.memory.set_fact("user", "Divyansh").await.unwrap();
        
        // Run inference
        let response = engine.process_request("Who am I?", InferenceOptions::default()).await.unwrap();
        
        // MockRuntime echoes the prompt. The prompt should now contain the injection.
        // Expected: "Mock response to: [Facts: user=Divyansh;]\n\nWho am I?"
        assert!(response.output.text.contains("user=Divyansh"));
    }
}