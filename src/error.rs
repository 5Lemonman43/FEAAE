use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum BotError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Market discovery error: {0}")]
    MarketDiscovery(String),

    #[error("Trading error: {0}")]
    Trading(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Parsing error: {0}")]
    Parse(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WebSocket connection error: {0}")]
    TungsteniteError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BotError>;
