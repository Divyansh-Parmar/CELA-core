use clap::{Parser, Subcommand};
use lie_core::{Engine, config::EngineConfig, runtime::InferenceOptions};
use lie_runtime_llamacpp::LlamaCppRuntime;
use lie_server::Server;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "lie")]
#[command(about = "Local AI Engine CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the engine in server mode
    Serve,
    /// Run a single inference (CLI mode)
    Run {
        #[arg(short, long)]
        prompt: String,
        
        #[arg(long)]
        max_tokens: Option<u32>,
        
        #[arg(long, default_value = "false")]
        enable_memory: bool,
    },
    /// Manage Memory
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    }
}

#[derive(Subcommand)]
enum MemoryAction {
    Set {
        key: String,
        value: String,
    },
    Summary {
        text: String,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    
    // In a real app, load config from file
    let mut config = EngineConfig::default();
    
    // Config Loading Mock-up (allow enabling memory via CLI args logic sort of)
    // Actually, for "Run" command, we can override config.
    // For "Memory" command, we assume default config (memory.json in cwd).
    // Ideally we load a config file. For v1, we just enable memory if requested.

    // Initialize Runtime
    let runtime = LlamaCppRuntime::new();
    
    match cli.command {
        Some(Commands::Serve) => {
            config.memory.enabled = true; // Enable memory for server by default or config?
            // Let's enable it if file exists? Or just true.
            config.memory.enabled = true;
            
            let engine = Engine::new(config, Box::new(runtime));
            let engine_arc = Arc::new(engine);
            engine_arc.init().await?;
            
            let server = Server::new(engine_arc);
            server.run().await?;
        }
        Some(Commands::Run { prompt, max_tokens, enable_memory }) => {
            config.memory.enabled = enable_memory;
            
            let engine = Engine::new(config, Box::new(runtime));
            let engine_arc = Arc::new(engine);
            engine_arc.init().await?;
            
            let mut options = InferenceOptions::default();
            if let Some(mt) = max_tokens {
                options.max_tokens = Some(mt);
            }

            let response = engine_arc.process_request(&prompt, options).await?;
            
            // Output valid JSON to stdout
            let json_output = serde_json::to_string_pretty(&response)?;
            println!("{}", json_output);
        }
        Some(Commands::Memory { action }) => {
            config.memory.enabled = true; // Must be enabled to write
            let engine = Engine::new(config, Box::new(runtime));
            
            match action {
                MemoryAction::Set { key, value } => {
                    engine.memory.set_fact(&key, &value).await?;
                    println!("Fact set: {} = {}", key, value);
                }
                MemoryAction::Summary { text } => {
                    engine.memory.update_summary(&text).await?;
                    println!("Summary updated.");
                }
            }
        }
        None => {
            println!("No command provided. Use --help");
        }
    }

    Ok(())
}
