mod api_client;
mod arbitrage;
mod config;
mod error;
mod types;
mod websocket;

use crate::api_client::PolymarketClient;
use crate::arbitrage::ArbitrageEngine;
use crate::config::Config;
use crate::error::Result;
use crate::types::{BotState, WsMessage};
use crate::websocket::WebSocketClient;
use log::{error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Starting Polymarket Arbitrage Bot");

    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    let client = Arc::new(PolymarketClient::new(config.polymarket_api_key.clone()));
    let state = Arc::new(Mutex::new(BotState::new()));

    info!("Discovering BTC down and SOL up 15-minute markets...");
    let btc_markets = client.discover_15m_btc_down_markets().await?;
    let sol_markets = client.discover_15m_sol_up_markets().await?;

    {
        let mut state_guard = state.lock().await;
        state_guard.btc_markets = btc_markets.clone();
        state_guard.sol_markets = sol_markets.clone();
    }

    info!(
        "Market discovery complete: {} BTC down markets, {} SOL up markets",
        btc_markets.len(),
        sol_markets.len()
    );

    if btc_markets.is_empty() {
        warn!("No BTC down markets found. Bot will continue monitoring for new markets.");
    } else {
        for market in &btc_markets {
            info!("BTC Market: {} - {}", market.id, market.question);
        }
    }

    if sol_markets.is_empty() {
        warn!("No SOL up markets found. Bot will continue monitoring for new markets.");
    } else {
        for market in &sol_markets {
            info!("SOL Market: {} - {}", market.id, market.question);
        }
    }

    let arbitrage_engine = Arc::new(ArbitrageEngine::new(
        config.clone(),
        client.clone(),
        state.clone(),
    ));

    let all_market_ids: Vec<String> = btc_markets
        .iter()
        .chain(sol_markets.iter())
        .map(|m| m.id.clone())
        .collect();

    let ws_state = state.clone();
    if !all_market_ids.is_empty() {
        info!("Starting WebSocket connection for real-time data...");
        
        let mut ws_client = WebSocketClient::new(config.polymarket_ws_url.clone());
        
        match ws_client.connect_and_subscribe(all_market_ids.clone()).await {
            Ok(mut ws_receiver) => {
                info!("WebSocket connected successfully");

                tokio::spawn(async move {
                    while let Some(message) = ws_receiver.recv().await {
                        match message {
                            WsMessage::Price { market, asset_id, price, .. } => {
                                info!("Price update for market {}: {}", market, price);
                                
                                let mut state = ws_state.lock().await;
                                
                                for btc_market in state.btc_markets.iter_mut() {
                                    if btc_market.id == market {
                                        if let Some(tokens) = &mut btc_market.tokens {
                                            for token in tokens.iter_mut() {
                                                if token.token_id == asset_id {
                                                    token.price = Some(price.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                for sol_market in state.sol_markets.iter_mut() {
                                    if sol_market.id == market {
                                        if let Some(tokens) = &mut sol_market.tokens {
                                            for token in tokens.iter_mut() {
                                                if token.token_id == asset_id {
                                                    token.price = Some(price.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            WsMessage::Book { market, asset_id, bids, asks, timestamp } => {
                                info!("Order book update for market {}: {} bids, {} asks", 
                                    market, bids.len(), asks.len());
                                
                                let mut state = ws_state.lock().await;
                                state.order_books.insert(
                                    format!("{}_{}", market, asset_id),
                                    crate::types::OrderBook {
                                        market: market.clone(),
                                        asset_id: asset_id.clone(),
                                        bids,
                                        asks,
                                        timestamp,
                                    },
                                );
                            }
                            WsMessage::TradeUpdate { market, price, size, .. } => {
                                info!("Trade update for market {}: {} @ {}", market, size, price);
                            }
                            WsMessage::LastTradePrice { market, price, .. } => {
                                info!("Last trade price for market {}: {}", market, price);
                            }
                            WsMessage::Market { market, .. } => {
                                info!("Market update: {}", market);
                            }
                        }
                    }
                    
                    warn!("WebSocket receiver closed");
                });
            }
            Err(e) => {
                warn!("Failed to connect WebSocket: {}. Continuing with polling mode.", e);
            }
        }
    }

    let arbitrage_engine_clone = arbitrage_engine.clone();
    let state_clone = state.clone();
    let client_clone = client.clone();
    
    tokio::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(30));
        
        loop {
            check_interval.tick().await;
            
            info!("Checking for new markets and arbitrage opportunities...");
            
            match client_clone.discover_15m_btc_down_markets().await {
                Ok(new_btc_markets) => {
                    let mut state = state_clone.lock().await;
                    if new_btc_markets.len() != state.btc_markets.len() {
                        info!("Updated BTC markets: {} markets", new_btc_markets.len());
                        state.btc_markets = new_btc_markets;
                    }
                }
                Err(e) => {
                    warn!("Failed to refresh BTC markets: {}", e);
                }
            }
            
            match client_clone.discover_15m_sol_up_markets().await {
                Ok(new_sol_markets) => {
                    let mut state = state_clone.lock().await;
                    if new_sol_markets.len() != state.sol_markets.len() {
                        info!("Updated SOL markets: {} markets", new_sol_markets.len());
                        state.sol_markets = new_sol_markets;
                    }
                }
                Err(e) => {
                    warn!("Failed to refresh SOL markets: {}", e);
                }
            }
            
            if let Err(e) = arbitrage_engine_clone.check_and_execute_arbitrage().await {
                error!("Error checking arbitrage: {}", e);
            }
            
            let state = state_clone.lock().await;
            info!(
                "Bot status - Positions: {}, Total trades: {}, Total PnL: {:.4}",
                state.positions.len(),
                state.trades.len(),
                state.total_pnl
            );
        }
    });

    info!("Bot is running. Press Ctrl+C to stop.");
    
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    
    info!("Shutting down gracefully...");
    
    let final_state = state.lock().await;
    info!("Final statistics:");
    info!("  Total trades executed: {}", final_state.trades.len());
    info!("  Active positions: {}", final_state.positions.len());
    info!("  Total PnL: {:.4}", final_state.total_pnl);
    
    info!("Bot stopped");
    Ok(())
}
