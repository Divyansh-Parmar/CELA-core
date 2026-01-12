use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

const SERVER_URL: &str = "http://127.0.0.1:8080";

#[derive(Serialize, Deserialize, Debug)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestLimits {
    max_tokens: Option<u32>,
    max_time_ms: Option<u64>,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CompletionRequest {
    prompt: String,
    limits: Option<RequestLimits>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Local AI Engine Reference Client ===");
    println!("Connecting to {}...", SERVER_URL);

    // 1. Health Check
    let client = reqwest::Client::new();
    let health_resp = client.get(format!("{}/v1/health", SERVER_URL))
        .send()
        .await;

    match health_resp {
        Ok(resp) => {
            if resp.status().is_success() {
                let health_json: HealthResponse = resp.json().await?;
                println!("Server OK: v{}", health_json.version);
            } else {
                println!("Server returned status: {}", resp.status());
                return Ok(())
            }
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
            println!("Ensure 'lie-cli serve' is running in another terminal.");
            return Ok(())
        }
    }

    println!("\nType your prompt. Special commands:");
    println!("  /limit <n>   Set max tokens (default 128)");
    println!("  /temp <n>    Set temperature (default 0.0)");
    println!("  /exit        Quit");

    // 2. REPL
    let mut rl = DefaultEditor::new()?;
    let mut current_max_tokens = 128;
    let mut current_temp = 0.0;

    loop {
        let readline = rl.readline(">>");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                rl.add_history_entry(line)?;

                if line.starts_with("/exit") {
                    break;
                } else if line.starts_with("/limit ") {
                    if let Ok(n) = line.trim_start_matches("/limit ").trim().parse::<u32>() {
                        current_max_tokens = n;
                        println!("Max tokens set to {}", n);
                    } else {
                        println!("Invalid number");
                    }
                    continue;
                } else if line.starts_with("/temp ") {
                    if let Ok(n) = line.trim_start_matches("/temp ").trim().parse::<f32>() {
                        current_temp = n;
                        println!("Temperature set to {}", n);
                    } else {
                        println!("Invalid number");
                    }
                    continue;
                }

                // 3. Send Request
                println!("Sending request...");
                let req = CompletionRequest {
                    prompt: line.to_string(),
                    limits: Some(RequestLimits {
                        max_tokens: Some(current_max_tokens),
                        max_time_ms: None,
                        temperature: Some(current_temp),
                    }),
                };

                let resp = client.post(format!("{}/v1/completion", SERVER_URL))
                    .json(&req)
                    .send()
                    .await;

                match resp {
                    Ok(r) => {
                        let json_body: serde_json::Value = r.json().await?;
                        // Pretty print the JSON contract
                        println!("{}", serde_json::to_string_pretty(&json_body)?);
                        
                        // Extract text for convenience
                        if let Some(text) = json_body.get("output").and_then(|o| o.get("text")).and_then(|t| t.as_str()) {
                            println!("\n--- Parsed Output ---\n{}
---------------------", text);
                        }
                    }
                    Err(e) => println!("Request failed: {}", e),
                }
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Exiting...");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
