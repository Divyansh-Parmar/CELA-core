use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub model: ModelConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub default_path: PathBuf,
    pub default_context_size: usize,
    pub default_gpu_layers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub enabled: bool,
    pub max_summary_chars: usize,
    pub max_kv_entries: usize,
    pub persistence_path: PathBuf,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                default_path: PathBuf::from("models/default.gguf"),
                default_context_size: 2048,
                default_gpu_layers: 0,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            memory: MemoryConfig::default(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_summary_chars: 1000,
            max_kv_entries: 50,
            persistence_path: PathBuf::from("memory.json"),
        }
    }
}