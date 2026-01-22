use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub condition_id: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub end_date_iso: String,
    #[serde(default)]
    pub game_start_time: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub outcomes: Vec<String>,
    #[serde(default)]
    pub outcome_prices: Vec<String>,
    #[serde(default)]
    pub volume: String,
    pub active: bool,
    pub closed: bool,
    #[serde(default)]
    pub market_type: String,
    pub tokens: Option<Vec<Token>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_id: String,
    pub outcome: String,
    pub price: Option<String>,
    #[serde(default)]
    pub winner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub market: String,
    pub asset_id: String,
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub market_id: String,
    pub outcome: String,
    pub side: TradeSide,
    pub amount: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub status: TradeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeStatus {
    Pending,
    Executed,
    PartiallyFilled,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Position {
    pub market_id: String,
    pub outcome: String,
    pub shares: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub pnl: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArbitrageOpportunity {
    pub btc_market: Market,
    pub sol_market: Market,
    pub btc_price: f64,
    pub sol_price: f64,
    pub expected_profit: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "market")]
    Market { market: String, timestamp: i64 },
    
    #[serde(rename = "book")]
    Book { 
        market: String, 
        asset_id: String,
        bids: Vec<Order>,
        asks: Vec<Order>,
        timestamp: i64,
    },
    
    #[serde(rename = "price")]
    Price {
        market: String,
        asset_id: String,
        price: String,
        timestamp: i64,
    },
    
    #[serde(rename = "trade")]
    TradeUpdate {
        market: String,
        asset_id: String,
        side: String,
        price: String,
        size: String,
        timestamp: i64,
    },
    
    #[serde(rename = "last_trade_price")]
    LastTradePrice {
        market: String,
        asset_id: String,
        price: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct WsSubscribeMessage {
    pub auth: Option<HashMap<String, String>>,
    pub markets: Vec<String>,
    pub assets_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BotState {
    pub positions: HashMap<String, Position>,
    pub trades: Vec<Trade>,
    pub btc_markets: Vec<Market>,
    pub sol_markets: Vec<Market>,
    pub order_books: HashMap<String, OrderBook>,
    pub total_pnl: f64,
}

impl BotState {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            trades: Vec::new(),
            btc_markets: Vec::new(),
            sol_markets: Vec::new(),
            order_books: HashMap::new(),
            total_pnl: 0.0,
        }
    }

    pub fn update_position(&mut self, trade: &Trade) {
        let key = format!("{}_{}", trade.market_id, trade.outcome);
        
        let position = self.positions.entry(key.clone()).or_insert(Position {
            market_id: trade.market_id.clone(),
            outcome: trade.outcome.clone(),
            shares: 0.0,
            average_price: 0.0,
            current_price: trade.price,
            pnl: 0.0,
        });

        match trade.side {
            TradeSide::Buy => {
                let total_cost = position.shares * position.average_price + trade.amount * trade.price;
                position.shares += trade.amount;
                position.average_price = if position.shares > 0.0 {
                    total_cost / position.shares
                } else {
                    0.0
                };
            }
            TradeSide::Sell => {
                position.shares -= trade.amount;
            }
        }

        position.current_price = trade.price;
        position.pnl = (position.current_price - position.average_price) * position.shares;
    }

    pub fn calculate_total_pnl(&mut self) {
        self.total_pnl = self.positions.values().map(|p| p.pnl).sum();
    }
}

impl Default for BotState {
    fn default() -> Self {
        Self::new()
    }
}
