use crate::error::{BotError, Result};
use crate::types::WsMessage;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde_json::json;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[allow(dead_code)]
pub struct WebSocketClient {
    url: String,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
}

impl WebSocketClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            reconnect_attempts: 0,
            max_reconnect_attempts: 10,
        }
    }

    pub async fn connect_and_subscribe(
        &mut self,
        market_ids: Vec<String>,
    ) -> Result<mpsc::UnboundedReceiver<WsMessage>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let url = self.url.clone();
        let markets = market_ids.clone();

        tokio::spawn(async move {
            let mut reconnect_delay = 1;

            loop {
                match Self::establish_connection(&url, &markets, tx.clone()).await {
                    Ok(_) => {
                        info!("WebSocket connection closed normally");
                        reconnect_delay = 1;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                    }
                }

                warn!(
                    "Reconnecting to WebSocket in {} seconds...",
                    reconnect_delay
                );
                sleep(Duration::from_secs(reconnect_delay)).await;

                reconnect_delay = std::cmp::min(reconnect_delay * 2, 60);
            }
        });

        Ok(rx)
    }

    async fn establish_connection(
        url: &str,
        market_ids: &[String],
        tx: mpsc::UnboundedSender<WsMessage>,
    ) -> Result<()> {
        info!("Connecting to WebSocket: {}", url);

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| BotError::WebSocket(format!("Connection failed: {}", e)))?;

        info!("WebSocket connected successfully");

        let (mut write, mut read) = ws_stream.split();

        let subscribe_msg = json!({
            "type": "subscribe",
            "market": market_ids,
        });

        write
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .map_err(|e| BotError::WebSocket(format!("Failed to send subscribe message: {}", e)))?;

        info!("Subscribed to {} markets", market_ids.len());

        let heartbeat_handle = tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                debug!("WebSocket heartbeat");
            }
        });

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    debug!("Received message: {}", text);

                    match serde_json::from_str::<WsMessage>(&text) {
                        Ok(ws_msg) => {
                            if tx.send(ws_msg).is_err() {
                                warn!("Receiver dropped, closing WebSocket");
                                break;
                            }
                        }
                        Err(e) => {
                            debug!("Failed to parse WebSocket message: {} - {}", e, text);
                        }
                    }
                }
                Ok(Message::Binary(data)) => {
                    debug!("Received binary message: {} bytes", data.len());
                }
                Ok(Message::Ping(data)) => {
                    debug!("Received ping, sending pong");
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        error!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Close(frame)) => {
                    info!("Received close frame: {:?}", frame);
                    break;
                }
                Ok(Message::Frame(_)) => {
                    debug!("Received raw frame");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        heartbeat_handle.abort();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn reset_reconnect_attempts(&mut self) {
        self.reconnect_attempts = 0;
    }
}
