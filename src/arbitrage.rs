use crate::api_client::PolymarketClient;
use crate::config::Config;
use crate::error::{BotError, Result};
use crate::types::{ArbitrageOpportunity, BotState, Market, Trade, TradeSide, TradeStatus};
use chrono::Utc;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ArbitrageEngine {
    config: Config,
    client: Arc<PolymarketClient>,
    state: Arc<Mutex<BotState>>,
}

impl ArbitrageEngine {
    pub fn new(
        config: Config,
        client: Arc<PolymarketClient>,
        state: Arc<Mutex<BotState>>,
    ) -> Self {
        Self {
            config,
            client,
            state,
        }
    }

    pub async fn detect_opportunities(&self) -> Result<Vec<ArbitrageOpportunity>> {
        let state = self.state.lock().await;
        let mut opportunities = Vec::new();

        for btc_market in &state.btc_markets {
            for sol_market in &state.sol_markets {
                if let Some(opportunity) = self.check_market_pair(btc_market, sol_market).await {
                    opportunities.push(opportunity);
                }
            }
        }

        Ok(opportunities)
    }

    async fn check_market_pair(
        &self,
        btc_market: &Market,
        sol_market: &Market,
    ) -> Option<ArbitrageOpportunity> {
        let btc_token = btc_market.tokens.as_ref()?.first()?;
        let sol_token = sol_market.tokens.as_ref()?.first()?;

        let btc_price = btc_token.price.as_ref()?.parse::<f64>().ok()?;
        let sol_price = sol_token.price.as_ref()?.parse::<f64>().ok()?;

        let combined_probability = btc_price + sol_price;
        
        if combined_probability < 1.0 {
            let expected_profit = 1.0 - combined_probability;

            if expected_profit > self.config.min_profit_threshold {
                info!(
                    "Arbitrage opportunity detected! BTC down: {:.4}, SOL up: {:.4}, Expected profit: {:.4}",
                    btc_price, sol_price, expected_profit
                );

                return Some(ArbitrageOpportunity {
                    btc_market: btc_market.clone(),
                    sol_market: sol_market.clone(),
                    btc_price,
                    sol_price,
                    expected_profit,
                    timestamp: Utc::now(),
                });
            }
        }

        None
    }

    pub async fn execute_arbitrage(&self, opportunity: &ArbitrageOpportunity) -> Result<Vec<Trade>> {
        info!(
            "Executing arbitrage: BTC down @ {:.4}, SOL up @ {:.4}, Expected profit: {:.4}",
            opportunity.btc_price, opportunity.sol_price, opportunity.expected_profit
        );

        let trade_amount = self.config.trade_amount;
        let mut trades = Vec::new();

        let btc_token = opportunity
            .btc_market
            .tokens
            .as_ref()
            .and_then(|t| t.first())
            .ok_or_else(|| BotError::Trading("BTC market has no tokens".to_string()))?;

        info!("Step 1: Buying BTC down position");
        let btc_trade = self
            .client
            .place_order(
                &opportunity.btc_market.id,
                &btc_token.token_id,
                TradeSide::Buy,
                trade_amount,
                opportunity.btc_price,
            )
            .await?;

        info!("BTC down trade executed: {:?}", btc_trade);
        trades.push(btc_trade.clone());

        {
            let mut state = self.state.lock().await;
            state.update_position(&btc_trade);
            state.trades.push(btc_trade);
        }

        if trades[0].status == TradeStatus::Executed {
            let sol_token = opportunity
                .sol_market
                .tokens
                .as_ref()
                .and_then(|t| t.first())
                .ok_or_else(|| BotError::Trading("SOL market has no tokens".to_string()))?;

            info!("Step 2: Buying SOL up position");
            let sol_trade = self
                .client
                .place_order(
                    &opportunity.sol_market.id,
                    &sol_token.token_id,
                    TradeSide::Buy,
                    trade_amount,
                    opportunity.sol_price,
                )
                .await?;

            info!("SOL up trade executed: {:?}", sol_trade);
            trades.push(sol_trade.clone());

            {
                let mut state = self.state.lock().await;
                state.update_position(&sol_trade);
                state.trades.push(sol_trade);
                state.calculate_total_pnl();
            }
        } else {
            warn!("BTC trade not fully executed, skipping SOL trade");
        }

        info!("Arbitrage execution complete: {} trades executed", trades.len());
        Ok(trades)
    }

    pub async fn update_market_prices(&self) -> Result<()> {
        let mut state = self.state.lock().await;

        for market in state.btc_markets.iter_mut() {
            if let Some(tokens) = &mut market.tokens {
                for token in tokens.iter_mut() {
                    match self.client.get_market_price(&market.id, &token.token_id).await {
                        Ok(price) => {
                            token.price = Some(price.to_string());
                        }
                        Err(e) => {
                            warn!("Failed to update price for market {}: {}", market.id, e);
                        }
                    }
                }
            }
        }

        for market in state.sol_markets.iter_mut() {
            if let Some(tokens) = &mut market.tokens {
                for token in tokens.iter_mut() {
                    match self.client.get_market_price(&market.id, &token.token_id).await {
                        Ok(price) => {
                            token.price = Some(price.to_string());
                        }
                        Err(e) => {
                            warn!("Failed to update price for market {}: {}", market.id, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn check_and_execute_arbitrage(&self) -> Result<()> {
        self.update_market_prices().await?;

        let opportunities = self.detect_opportunities().await?;

        if opportunities.is_empty() {
            info!("No arbitrage opportunities found");
            return Ok(());
        }

        info!("Found {} arbitrage opportunities", opportunities.len());

        for opportunity in opportunities.iter().take(1) {
            match self.execute_arbitrage(opportunity).await {
                Ok(trades) => {
                    info!("Successfully executed arbitrage with {} trades", trades.len());
                }
                Err(e) => {
                    warn!("Failed to execute arbitrage: {}", e);
                }
            }
        }

        Ok(())
    }
}
