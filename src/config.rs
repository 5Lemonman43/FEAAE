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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_config_from_env_with_defaults() {
        std::env::remove_var("POLYMARKET_API_KEY");
        std::env::remove_var("MIN_PROFIT_THRESHOLD");
        std::env::remove_var("MAX_POSITION_SIZE");
        std::env::remove_var("TRADE_AMOUNT");
        std::env::remove_var("POLYMARKET_WS_URL");
        
        let config = Config::from_env().unwrap();
        assert!(config.polymarket_api_key.is_none());
        assert!((config.min_profit_threshold - 0.02).abs() < 0.001);
        assert!((config.max_position_size - 100.0).abs() < 0.1);
        assert!((config.trade_amount - 10.0).abs() < 0.1);
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_custom_values() {
        std::env::remove_var("POLYMARKET_WS_URL");
        std::env::set_var("MIN_PROFIT_THRESHOLD", "0.05");
        std::env::set_var("MAX_POSITION_SIZE", "200.0");
        std::env::set_var("TRADE_AMOUNT", "25.0");
        
        let config = Config::from_env().unwrap();
        assert!((config.min_profit_threshold - 0.05).abs() < 0.001);
        assert!((config.max_position_size - 200.0).abs() < 0.1);
        assert!((config.trade_amount - 25.0).abs() < 0.1);
        
        std::env::remove_var("MIN_PROFIT_THRESHOLD");
        std::env::remove_var("MAX_POSITION_SIZE");
        std::env::remove_var("TRADE_AMOUNT");
    }

    #[test]
    #[serial]
    fn test_config_invalid_threshold() {
        std::env::set_var("MIN_PROFIT_THRESHOLD", "invalid");
        let result = Config::from_env();
        assert!(result.is_err());
        std::env::remove_var("MIN_PROFIT_THRESHOLD");
    }
}
