use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}
