use crate::error::{BotError, Result};
use crate::types::{Market, OrderBook, Trade, TradeSide, TradeStatus};
use chrono::Utc;
use log::{debug, info, warn};
use reqwest::Client;
use serde_json::{json, Value};

const POLYMARKET_API_BASE: &str = "https://clob.polymarket.com";
const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";

pub struct PolymarketClient {
    client: Client,
    api_key: Option<String>,
}

impl PolymarketClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn discover_markets(&self, search_term: &str) -> Result<Vec<Market>> {
        info!("Discovering markets with search term: {}", search_term);
        
        let url = format!("{}/markets", GAMMA_API_BASE);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(BotError::Api(format!(
                "Failed to fetch markets: {}",
                response.status()
            )));
        }

        let markets: Vec<Market> = response.json().await?;
        
        let filtered_markets: Vec<Market> = markets
            .into_iter()
            .filter(|m| {
                let question_lower = m.question.to_lowercase();
                let description_lower = m.description.to_lowercase();
                let is_active = m.active && !m.closed;
                let matches_search = question_lower.contains(search_term) 
                    || description_lower.contains(search_term);
                
                is_active && matches_search
            })
            .collect();

        info!("Found {} markets matching '{}'", filtered_markets.len(), search_term);
        Ok(filtered_markets)
    }

    pub async fn discover_15m_btc_down_markets(&self) -> Result<Vec<Market>> {
        info!("Discovering 15m BTC down markets");
        
        let markets = self.discover_markets("btc").await?;
        
        let btc_down_markets: Vec<Market> = markets
            .into_iter()
            .filter(|m| {
                let question_lower = m.question.to_lowercase();
                let is_15m = question_lower.contains("15") 
                    || question_lower.contains("fifteen")
                    || question_lower.contains("15 minute")
                    || question_lower.contains("15m");
                let is_down = question_lower.contains("down") 
                    || question_lower.contains("lower")
                    || question_lower.contains("decrease")
                    || question_lower.contains("fall");
                let is_btc = question_lower.contains("btc") 
                    || question_lower.contains("bitcoin");
                
                is_15m && is_down && is_btc
            })
            .collect();

        info!("Found {} 15m BTC down markets", btc_down_markets.len());
        Ok(btc_down_markets)
    }

    pub async fn discover_15m_sol_up_markets(&self) -> Result<Vec<Market>> {
        info!("Discovering 15m SOL up markets");
        
        let markets = self.discover_markets("sol").await?;
        
        let sol_up_markets: Vec<Market> = markets
            .into_iter()
            .filter(|m| {
                let question_lower = m.question.to_lowercase();
                let is_15m = question_lower.contains("15") 
                    || question_lower.contains("fifteen")
                    || question_lower.contains("15 minute")
                    || question_lower.contains("15m");
                let is_up = question_lower.contains("up") 
                    || question_lower.contains("higher")
                    || question_lower.contains("increase")
                    || question_lower.contains("rise");
                let is_sol = question_lower.contains("sol") 
                    || question_lower.contains("solana");
                
                is_15m && is_up && is_sol
            })
            .collect();

        info!("Found {} 15m SOL up markets", sol_up_markets.len());
        Ok(sol_up_markets)
    }

    pub async fn get_order_book(&self, market_id: &str, asset_id: &str) -> Result<OrderBook> {
        debug!("Fetching order book for market: {}, asset: {}", market_id, asset_id);
        
        let url = format!("{}/book", POLYMARKET_API_BASE);
        
        let response = self
            .client
            .get(&url)
            .query(&[("token_id", asset_id)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(BotError::Api(format!(
                "Failed to fetch order book: {}",
                response.status()
            )));
        }

        let json: Value = response.json().await?;
        
        let order_book = OrderBook {
            market: market_id.to_string(),
            asset_id: asset_id.to_string(),
            bids: serde_json::from_value(json["bids"].clone()).unwrap_or_default(),
            asks: serde_json::from_value(json["asks"].clone()).unwrap_or_default(),
            timestamp: Utc::now().timestamp(),
        };

        Ok(order_book)
    }

    pub async fn place_order(
        &self,
        market_id: &str,
        asset_id: &str,
        side: TradeSide,
        amount: f64,
        price: f64,
    ) -> Result<Trade> {
        info!(
            "Placing {} order for market: {}, amount: {}, price: {}",
            match side {
                TradeSide::Buy => "BUY",
                TradeSide::Sell => "SELL",
            },
            market_id,
            amount,
            price
        );

        if self.api_key.is_none() {
            warn!("API key not configured - simulating order execution");
            return Ok(Trade {
                id: format!("sim_{}", Utc::now().timestamp()),
                market_id: market_id.to_string(),
                outcome: asset_id.to_string(),
                side,
                amount,
                price,
                timestamp: Utc::now(),
                status: TradeStatus::Executed,
            });
        }

        let url = format!("{}/order", POLYMARKET_API_BASE);
        
        let order_payload = json!({
            "market": market_id,
            "asset_id": asset_id,
            "side": match side {
                TradeSide::Buy => "BUY",
                TradeSide::Sell => "SELL",
            },
            "size": amount.to_string(),
            "price": price.to_string(),
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.as_ref().unwrap()))
            .json(&order_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BotError::Trading(format!(
                "Failed to place order: {}",
                error_text
            )));
        }

        let result: Value = response.json().await?;
        
        Ok(Trade {
            id: result["order_id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            market_id: market_id.to_string(),
            outcome: asset_id.to_string(),
            side,
            amount,
            price,
            timestamp: Utc::now(),
            status: TradeStatus::Executed,
        })
    }

    pub async fn get_market_price(&self, market_id: &str, asset_id: &str) -> Result<f64> {
        debug!("Fetching price for market: {}, asset: {}", market_id, asset_id);
        
        let order_book = self.get_order_book(market_id, asset_id).await?;
        
        if let Some(best_ask) = order_book.asks.first() {
            return best_ask.price.parse::<f64>()
                .map_err(|_| BotError::Parse("Failed to parse price".to_string()));
        }
        
        if let Some(best_bid) = order_book.bids.first() {
            return best_bid.price.parse::<f64>()
                .map_err(|_| BotError::Parse("Failed to parse price".to_string()));
        }
        
        Err(BotError::Api("No price data available".to_string()))
    }
}
