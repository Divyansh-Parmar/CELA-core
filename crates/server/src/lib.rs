use axum::{
    extract::{State, Json},
    routing::{post, get},
    Router,
};
use lie_core::{Engine, EngineResponse, runtime::InferenceOptions, OutputContent, runtime::Usage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::net::SocketAddr;
use anyhow::Result;

#[derive(Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub limits: Option<RequestLimits>,
}

#[derive(Serialize, Deserialize)]
pub struct RequestLimits {
    pub max_tokens: Option<u32>,
    pub max_time_ms: Option<u64>,
    pub temperature: Option<f32>,
}

pub struct Server {
    engine: Arc<Engine>,
}

impl Server {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    pub async fn run(&self) -> Result<()> {
        let app = Router::new()
            .route("/v1/health", get(health_check))
            .route("/v1/completion", post(handle_completion))
            .with_state(self.engine.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        println!("Server listening on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "lie-server",
        "version": "1.0.0"
    }))
}

fn validate_request(payload: &CompletionRequest) -> Result<InferenceOptions, String> {
    if payload.prompt.trim().is_empty() {
        return Err("Validation Error: Prompt cannot be empty".to_string());
    }

    let mut options = InferenceOptions::default();
    if let Some(limits) = &payload.limits {
        if let Some(mt) = limits.max_tokens {
            if mt == 0 || mt > 8192 {
                 return Err("Validation Error: max_tokens must be between 1 and 8192".to_string());
            }
            options.max_tokens = Some(mt);
        }
        
        if let Some(mtm) = limits.max_time_ms {
             if mtm > 300_000 {
                 return Err("Validation Error: max_time_ms cannot exceed 300000".to_string());
             }
             options.max_time_ms = Some(mtm);
        }

        if let Some(temp) = limits.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err("Validation Error: temperature must be between 0.0 and 2.0".to_string());
            }
            options.temperature = Some(temp);
        }
    }
    Ok(options)
}

async fn handle_completion(
    State(engine): State<Arc<Engine>>,
    Json(payload): Json<CompletionRequest>,
) -> Json<EngineResponse> {
    
    // 1. Validation
    let options = match validate_request(&payload) {
        Ok(opts) => opts,
        Err(e) => return Json(EngineResponse {
            status: "error".to_string(),
            intent: None,
            output: OutputContent { text: "".to_string() },
            usage: Usage::default(),
            error: Some(e),
        }),
    };

    // 2. Processing
    match engine.process_request(&payload.prompt, options).await {
        Ok(response) => Json(response),
        Err(e) => {
            Json(EngineResponse {
                status: "error".to_string(),
                intent: None,
                output: OutputContent { text: "".to_string() },
                usage: Usage::default(),
                error: Some(format!("Runtime Error: {}", e)),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_prompt() {
        let req = CompletionRequest { prompt: "   ".to_string(), limits: None };
        assert!(validate_request(&req).is_err());
    }

    #[test]
    fn test_validation_invalid_limits() {
        let req = CompletionRequest { 
            prompt: "Hi".to_string(), 
            limits: Some(RequestLimits { max_tokens: Some(9000), max_time_ms: None, temperature: None }) 
        };
        assert!(validate_request(&req).is_err());
    }

    #[test]
    fn test_validation_valid() {
        let req = CompletionRequest { 
            prompt: "Hi".to_string(), 
            limits: Some(RequestLimits { max_tokens: Some(10), max_time_ms: None, temperature: Some(0.5) }) 
        };
        assert!(validate_request(&req).is_ok());
    }
}
