use crate::error::{BotError, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub polymarket_api_key: Option<String>,
    pub polymarket_secret: Option<String>,
    pub polymarket_private_key: Option<String>,
    pub polymarket_ws_url: String,
    pub min_profit_threshold: f64,
    pub max_position_size: f64,
    pub trade_amount: f64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let polymarket_api_key = env::var("POLYMARKET_API_KEY").ok();
        let polymarket_secret = env::var("POLYMARKET_SECRET").ok();
        let polymarket_private_key = env::var("POLYMARKET_PRIVATE_KEY").ok();
        
        let polymarket_ws_url = env::var("POLYMARKET_WS_URL")
            .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string());
        
        let min_profit_threshold = env::var("MIN_PROFIT_THRESHOLD")
            .unwrap_or_else(|_| "0.02".to_string())
            .parse()
            .map_err(|_| BotError::Config("Invalid MIN_PROFIT_THRESHOLD".to_string()))?;
        
        let max_position_size = env::var("MAX_POSITION_SIZE")
            .unwrap_or_else(|_| "100.0".to_string())
            .parse()
            .map_err(|_| BotError::Config("Invalid MAX_POSITION_SIZE".to_string()))?;
        
        let trade_amount = env::var("TRADE_AMOUNT")
            .unwrap_or_else(|_| "10.0".to_string())
            .parse()
            .map_err(|_| BotError::Config("Invalid TRADE_AMOUNT".to_string()))?;

        Ok(Config {
            polymarket_api_key,
            polymarket_secret,
            polymarket_private_key,
            polymarket_ws_url,
            min_profit_threshold,
            max_position_size,
            trade_amount,
        })
    }
}
