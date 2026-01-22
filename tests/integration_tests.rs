use polymarket_arbitrage_bot::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_bot_state_creation() {
    let state = types::BotState::new();
    assert_eq!(state.positions.len(), 0);
    assert_eq!(state.trades.len(), 0);
    assert_eq!(state.total_pnl, 0.0);
}

#[tokio::test]
async fn test_trade_execution_flow() {
    let state = Arc::new(Mutex::new(types::BotState::new()));
    
    let trade = types::Trade {
        id: "test_trade_1".to_string(),
        market_id: "btc_market_1".to_string(),
        outcome: "down".to_string(),
        side: types::TradeSide::Buy,
        amount: 10.0,
        price: 0.45,
        timestamp: chrono::Utc::now(),
        status: types::TradeStatus::Executed,
    };
    
    {
        let mut state_guard = state.lock().await;
        state_guard.update_position(&trade);
        state_guard.trades.push(trade);
    }
    
    let state_guard = state.lock().await;
    assert_eq!(state_guard.trades.len(), 1);
    assert_eq!(state_guard.positions.len(), 1);
}

#[tokio::test]
async fn test_arbitrage_detection_logic() {
    let btc_price: f64 = 0.45;
    let sol_price: f64 = 0.48;
    let combined = btc_price + sol_price;
    let expected_profit = 1.0 - combined;
    
    assert!(expected_profit > 0.0, "Arbitrage opportunity should exist");
    assert!((expected_profit - 0.07).abs() < 0.001, "Expected profit should be approximately 0.07, got {}", expected_profit);
}

#[tokio::test]
async fn test_position_pnl_calculation() {
    let mut state = types::BotState::new();
    
    let buy_trade = types::Trade {
        id: "buy1".to_string(),
        market_id: "market1".to_string(),
        outcome: "Yes".to_string(),
        side: types::TradeSide::Buy,
        amount: 10.0,
        price: 0.50,
        timestamp: chrono::Utc::now(),
        status: types::TradeStatus::Executed,
    };
    state.update_position(&buy_trade);
    
    let price_update_trade = types::Trade {
        id: "update1".to_string(),
        market_id: "market1".to_string(),
        outcome: "Yes".to_string(),
        side: types::TradeSide::Buy,
        amount: 0.0,
        price: 0.60,
        timestamp: chrono::Utc::now(),
        status: types::TradeStatus::Executed,
    };
    state.update_position(&price_update_trade);
    
    state.calculate_total_pnl();
    
    assert!((state.total_pnl - 1.0).abs() < 0.01, "PnL should be approximately 1.0, got {}", state.total_pnl);
}

#[tokio::test]
async fn test_market_filtering_logic() {
    let btc_market = types::Market {
        id: "btc_15m_down_1".to_string(),
        question: "Will BTC go down in the next 15 minutes?".to_string(),
        condition_id: "cond1".to_string(),
        slug: "btc-15m-down".to_string(),
        end_date_iso: "2025-01-22T12:00:00Z".to_string(),
        game_start_time: "2025-01-22T11:45:00Z".to_string(),
        description: "BTC 15 minute down prediction market".to_string(),
        outcomes: vec!["Yes".to_string(), "No".to_string()],
        outcome_prices: vec!["0.45".to_string(), "0.55".to_string()],
        volume: "10000".to_string(),
        active: true,
        closed: false,
        market_type: "binary".to_string(),
        tokens: Some(vec![types::Token {
            token_id: "token_btc_down".to_string(),
            outcome: "Yes".to_string(),
            price: Some("0.45".to_string()),
            winner: false,
        }]),
    };
    
    let question_lower = btc_market.question.to_lowercase();
    let is_15m = question_lower.contains("15");
    let is_down = question_lower.contains("down");
    let is_btc = question_lower.contains("btc");
    
    assert!(is_15m, "Market should be identified as 15m");
    assert!(is_down, "Market should be identified as down");
    assert!(is_btc, "Market should be identified as BTC");
    assert!(btc_market.active, "Market should be active");
    assert!(!btc_market.closed, "Market should not be closed");
}

#[tokio::test]
async fn test_sol_market_filtering() {
    let sol_market = types::Market {
        id: "sol_15m_up_1".to_string(),
        question: "Will SOL go up in the next 15 minutes?".to_string(),
        condition_id: "cond2".to_string(),
        slug: "sol-15m-up".to_string(),
        end_date_iso: "2025-01-22T12:00:00Z".to_string(),
        game_start_time: "2025-01-22T11:45:00Z".to_string(),
        description: "SOL 15 minute up prediction market".to_string(),
        outcomes: vec!["Yes".to_string(), "No".to_string()],
        outcome_prices: vec!["0.48".to_string(), "0.52".to_string()],
        volume: "10000".to_string(),
        active: true,
        closed: false,
        market_type: "binary".to_string(),
        tokens: Some(vec![types::Token {
            token_id: "token_sol_up".to_string(),
            outcome: "Yes".to_string(),
            price: Some("0.48".to_string()),
            winner: false,
        }]),
    };
    
    let question_lower = sol_market.question.to_lowercase();
    let is_15m = question_lower.contains("15");
    let is_up = question_lower.contains("up");
    let is_sol = question_lower.contains("sol");
    
    assert!(is_15m, "Market should be identified as 15m");
    assert!(is_up, "Market should be identified as up");
    assert!(is_sol, "Market should be identified as SOL");
    assert!(sol_market.active, "Market should be active");
}

#[tokio::test]
async fn test_sequential_trade_execution() {
    let mut state = types::BotState::new();
    
    let btc_trade = types::Trade {
        id: "btc_trade_1".to_string(),
        market_id: "btc_market_1".to_string(),
        outcome: "down".to_string(),
        side: types::TradeSide::Buy,
        amount: 10.0,
        price: 0.45,
        timestamp: chrono::Utc::now(),
        status: types::TradeStatus::Executed,
    };
    state.update_position(&btc_trade);
    state.trades.push(btc_trade.clone());
    
    assert_eq!(btc_trade.status, types::TradeStatus::Executed);
    
    let sol_trade = types::Trade {
        id: "sol_trade_1".to_string(),
        market_id: "sol_market_1".to_string(),
        outcome: "up".to_string(),
        side: types::TradeSide::Buy,
        amount: 10.0,
        price: 0.48,
        timestamp: chrono::Utc::now(),
        status: types::TradeStatus::Executed,
    };
    state.update_position(&sol_trade);
    state.trades.push(sol_trade);
    
    assert_eq!(state.trades.len(), 2);
    assert_eq!(state.positions.len(), 2);
}

#[tokio::test]
async fn test_profit_threshold_check() {
    let min_profit_threshold = 0.02;
    
    let btc_price = 0.45;
    let sol_price = 0.48;
    let expected_profit = 1.0 - (btc_price + sol_price);
    
    assert!(expected_profit > min_profit_threshold, 
        "Expected profit {} should exceed threshold {}", expected_profit, min_profit_threshold);
}

#[tokio::test]
async fn test_below_threshold_rejection() {
    let min_profit_threshold = 0.10;
    
    let btc_price = 0.45;
    let sol_price = 0.48;
    let expected_profit = 1.0 - (btc_price + sol_price);
    
    assert!(expected_profit < min_profit_threshold, 
        "Expected profit {} should be below threshold {}", expected_profit, min_profit_threshold);
}

#[test]
fn test_websocket_message_parsing() {
    let price_msg = r#"{"type":"price","market":"test_market","asset_id":"asset1","price":"0.55","timestamp":1234567890}"#;
    let parsed: Result<types::WsMessage, _> = serde_json::from_str(price_msg);
    assert!(parsed.is_ok(), "Should parse price message");
    
    match parsed.unwrap() {
        types::WsMessage::Price { market, price, .. } => {
            assert_eq!(market, "test_market");
            assert_eq!(price, "0.55");
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_order_book_message_parsing() {
    let book_msg = r#"{"type":"book","market":"test_market","asset_id":"asset1","bids":[{"price":"0.48","size":"100"}],"asks":[{"price":"0.52","size":"100"}],"timestamp":1234567890}"#;
    let parsed: Result<types::WsMessage, _> = serde_json::from_str(book_msg);
    assert!(parsed.is_ok(), "Should parse book message");
    
    match parsed.unwrap() {
        types::WsMessage::Book { market, bids, asks, .. } => {
            assert_eq!(market, "test_market");
            assert_eq!(bids.len(), 1);
            assert_eq!(asks.len(), 1);
            assert_eq!(bids[0].price, "0.48");
            assert_eq!(asks[0].price, "0.52");
        }
        _ => panic!("Wrong message type"),
    }
}

#[tokio::test]
async fn test_error_handling() {
    use polymarket_arbitrage_bot::error::BotError;
    
    let api_error = BotError::Api("Test API error".to_string());
    assert!(format!("{}", api_error).contains("API error"));
    
    let trading_error = BotError::Trading("Test trading error".to_string());
    assert!(format!("{}", trading_error).contains("Trading error"));
    
    let config_error = BotError::Config("Invalid config".to_string());
    assert!(format!("{}", config_error).contains("Configuration error"));
}

#[tokio::test]
async fn test_state_thread_safety() {
    let state = Arc::new(Mutex::new(types::BotState::new()));
    let mut handles = vec![];
    
    for i in 0..10 {
        let state_clone = Arc::clone(&state);
        let handle = tokio::spawn(async move {
            let trade = types::Trade {
                id: format!("trade_{}", i),
                market_id: format!("market_{}", i),
                outcome: "Yes".to_string(),
                side: types::TradeSide::Buy,
                amount: 10.0,
                price: 0.5,
                timestamp: chrono::Utc::now(),
                status: types::TradeStatus::Executed,
            };
            
            let mut state_guard = state_clone.lock().await;
            state_guard.update_position(&trade);
            state_guard.trades.push(trade);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let state_guard = state.lock().await;
    assert_eq!(state_guard.trades.len(), 10);
    assert!(state_guard.positions.len() <= 10);
}
