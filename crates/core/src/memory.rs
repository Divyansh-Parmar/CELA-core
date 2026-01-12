use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::EngineError;
use crate::config::MemoryConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct MemoryData {
    summary: String,
    kv_store: HashMap<String, String>,
}

pub struct MemoryManager {
    config: MemoryConfig,
    data: Arc<RwLock<MemoryData>>,
}

impl MemoryManager {
    pub fn new(config: MemoryConfig) -> Self {
        let data = if config.enabled && config.persistence_path.exists() {
            // Try to load
            match fs::read_to_string(&config.persistence_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => MemoryData::default(),
            }
        } else {
            MemoryData::default()
        };

        Self {
            config,
            data: Arc::new(RwLock::new(data)),
        }
    }

    pub async fn get_injection_text(&self) -> String {
        if !self.config.enabled {
            return String::new();
        }

        let data = self.data.read().await;
        let mut injection = String::new();

        if !data.summary.is_empty() {
            injection.push_str(&format!("[Summary: {}]\n", data.summary));
        }

        if !data.kv_store.is_empty() {
            injection.push_str("[Facts:");
            for (k, v) in &data.kv_store {
                injection.push_str(&format!(" {}={};", k, v));
            }
            injection.push_str("]\n");
        }
        
        if !injection.is_empty() {
             injection.push('\n'); // Separator
        }

        injection
    }

    pub async fn update_summary(&self, text: &str) -> Result<(), EngineError> {
        if !self.config.enabled { return Ok(()); } 
        
        let mut data = self.data.write().await;
        
        // Simple append for v1, enforcing limit
        let mut new_summary = data.summary.clone();
        if !new_summary.is_empty() {
            new_summary.push_str(" ");
        }
        new_summary.push_str(text);

        // Truncate from beginning if too long (Rolling window)
        if new_summary.len() > self.config.max_summary_chars {
            let start = new_summary.len() - self.config.max_summary_chars;
            new_summary = new_summary[start..].to_string();
        }
        
        data.summary = new_summary;
        self.save(&data)?;
        Ok(())
    }

    pub async fn set_fact(&self, key: &str, value: &str) -> Result<(), EngineError> {
        if !self.config.enabled { return Ok(()); } 

        let mut data = self.data.write().await;
        
        if data.kv_store.len() >= self.config.max_kv_entries && !data.kv_store.contains_key(key) {
             return Err(EngineError::Config("Memory KV limit reached".to_string()));
        }

        data.kv_store.insert(key.to_string(), value.to_string());
        self.save(&data)?;
        Ok(())
    }

    fn save(&self, data: &MemoryData) -> Result<(), EngineError> {
        if self.config.enabled {
            let json = serde_json::to_string_pretty(data)
                .map_err(|e| EngineError::Unknown(format!("Serialization error: {}\n", e)))?;
            fs::write(&self.config.persistence_path, json)?;
        }
        Ok(())
    }
}