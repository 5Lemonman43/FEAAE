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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_state_new() {
        let state = BotState::new();
        assert_eq!(state.positions.len(), 0);
        assert_eq!(state.trades.len(), 0);
        assert_eq!(state.btc_markets.len(), 0);
        assert_eq!(state.sol_markets.len(), 0);
        assert_eq!(state.total_pnl, 0.0);
    }

    #[test]
    fn test_update_position_buy() {
        let mut state = BotState::new();
        let trade = Trade {
            id: "test1".to_string(),
            market_id: "market1".to_string(),
            outcome: "Yes".to_string(),
            side: TradeSide::Buy,
            amount: 10.0,
            price: 0.5,
            timestamp: Utc::now(),
            status: TradeStatus::Executed,
        };

        state.update_position(&trade);
        let key = "market1_Yes";
        assert!(state.positions.contains_key(key));
        let position = state.positions.get(key).unwrap();
        assert_eq!(position.shares, 10.0);
        assert_eq!(position.average_price, 0.5);
    }

    #[test]
    fn test_update_position_multiple_buys() {
        let mut state = BotState::new();
        
        let trade1 = Trade {
            id: "test1".to_string(),
            market_id: "market1".to_string(),
            outcome: "Yes".to_string(),
            side: TradeSide::Buy,
            amount: 10.0,
            price: 0.5,
            timestamp: Utc::now(),
            status: TradeStatus::Executed,
        };
        state.update_position(&trade1);

        let trade2 = Trade {
            id: "test2".to_string(),
            market_id: "market1".to_string(),
            outcome: "Yes".to_string(),
            side: TradeSide::Buy,
            amount: 10.0,
            price: 0.6,
            timestamp: Utc::now(),
            status: TradeStatus::Executed,
        };
        state.update_position(&trade2);

        let key = "market1_Yes";
        let position = state.positions.get(key).unwrap();
        assert_eq!(position.shares, 20.0);
        assert_eq!(position.average_price, 0.55);
    }

    #[test]
    fn test_calculate_total_pnl() {
        let mut state = BotState::new();
        
        let trade1 = Trade {
            id: "test1".to_string(),
            market_id: "market1".to_string(),
            outcome: "Yes".to_string(),
            side: TradeSide::Buy,
            amount: 10.0,
            price: 0.5,
            timestamp: Utc::now(),
            status: TradeStatus::Executed,
        };
        state.update_position(&trade1);

        let trade2 = Trade {
            id: "test2".to_string(),
            market_id: "market1".to_string(),
            outcome: "Yes".to_string(),
            side: TradeSide::Buy,
            amount: 0.0,
            price: 0.6,
            timestamp: Utc::now(),
            status: TradeStatus::Executed,
        };
        state.update_position(&trade2);

        state.calculate_total_pnl();
        assert!((state.total_pnl - 1.0).abs() < 0.01, "PnL should be approximately 1.0, got {}", state.total_pnl);
    }

    #[test]
    fn test_market_serialization() {
        let market = Market {
            id: "test_market".to_string(),
            question: "Test question?".to_string(),
            condition_id: "cond1".to_string(),
            slug: "test-slug".to_string(),
            end_date_iso: "2025-01-22".to_string(),
            game_start_time: "2025-01-22T10:00:00Z".to_string(),
            description: "Test description".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            outcome_prices: vec!["0.5".to_string(), "0.5".to_string()],
            volume: "1000".to_string(),
            active: true,
            closed: false,
            market_type: "binary".to_string(),
            tokens: Some(vec![Token {
                token_id: "token1".to_string(),
                outcome: "Yes".to_string(),
                price: Some("0.5".to_string()),
                winner: false,
            }]),
        };

        let json = serde_json::to_string(&market).unwrap();
        let deserialized: Market = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test_market");
        assert_eq!(deserialized.active, true);
    }

    #[test]
    fn test_ws_message_deserialization() {
        let json = r#"{"type":"price","market":"test_market","asset_id":"asset1","price":"0.55","timestamp":1234567890}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Price { market, price, .. } => {
                assert_eq!(market, "test_market");
                assert_eq!(price, "0.55");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
