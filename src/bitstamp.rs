//! # Bitstamp WebSocket Module
//!
//! This module provides functionality for interacting with the Bitstamp cryptocurrency exchange.
//!
//! It includes utilities for subscribing to and processing real-time order book updates via
//! WebSocket, as well as requesting snapshots of the order book via HTTP. The module is designed
//! to work with a `PxQtLadderSender` channel for transmitting market data updates to other parts
//! of the application.
//!
//! # Features
//!
//! - **WebSocket Connection**: Establishes a WebSocket connection to the Bitstamp API to receive
//!   real-time order book updates.
//! - **Order Book Parsing**: Parses incoming order book data and extracts top N bids and asks.
//! - **Subscription Management**: Handles subscription and unsubscription to specific trading
//!   pairs on the WebSocket.
//! - **Error Handling**: Logs and handles various errors, including malformed messages and
//!   connection issues.
//! - **HTTP Snapshot**: Provides a function to request a snapshot of the order book via HTTP.
//!
//! # Key Components
//!
//! - **Structs**:
//!   - `Heartbeat`: Represents a heartbeat message from the WebSocket.
//!   - `Subscription`: Represents a subscription or unsubscription request.
//!   - `InStreamMessage`: Represents a generic incoming WebSocket message.
//!   - `OrderBook` and `OrderBookData`: Represent the structure of order book data.
//!
//! - **Constants**:
//!   - `BITSTAMP_ORDERBOOK_DATA_OFFSET`: Offset for extracting order book data from incoming messages.
//!   - `DATA_PREFIX`: Prefix used to identify messages containing order book data.
//!
//! - **Functions**:
//!   - `connect_websocket`: Connects to the WebSocket API, subscribes to a trading pair, processes
//!     incoming messages, and sends parsed order book data through a channel.
//!   - `request_snapshot`: Requests a snapshot of the order book via HTTP.
//!   - `skip_to_orderbook_data`: Utility function to extract order book data from a JSON string.
//!
//! # Usage
//!
//! To use this module, you need to:
//! 1. Establish a WebSocket connection using `connect_websocket`.
//! 2. Subscribe to a trading pair and process incoming order book updates.
//! 3. Optionally, request a snapshot of the order book using `request_snapshot`.
//!
//! # Example
//!
//! ```rust
//! use bitstamp::connect_websocket;
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (tx, rx) = mpsc::channel(100);
//!     let connector = tokio_tungstenite::Connector::default();
//!     let url = "wss://ws.bitstamp.net";
//!     let symbol = "btcusd";
//!     let top_n = 10;
//!
//!     if let Err(e) = connect_websocket(connector, url, symbol, &tx, top_n).await {
//!         eprintln!("Error: {}", e);
//!     }
//! }
//! ```
//!
//! # Notes
//!
//! - This module relies on the `tokio-tungstenite` crate for WebSocket communication and the
//!   `serde` crate for JSON serialization and deserialization.
//! - Ensure proper error handling and reconnection logic in production environments.

use super::data_tunnel::PxQtLadderSender;
use super::data_types::{PxQtBook, PxQtLadder};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::protocol::Message};
use tracing;

#[derive(Debug, Serialize, Deserialize)]
pub struct Heartbeat {
    pub event: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub event: String,
    pub data: SubscriptionData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub channel: String,
}

#[derive(Debug, Deserialize)]
pub struct InStreamMessage {
    pub event: Option<String>,
    pub channel: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OrderBook {
    pub data: OrderBookData,
    pub event: String,
    pub channel: String,
}
#[derive(Debug, Deserialize)]
pub struct OrderBookData {
    pub timestamp: String,
    pub microtimestamp: String,
    pub bids: PxQtLadder,
    pub asks: PxQtLadder,
}
impl PxQtBook for OrderBook {
    fn get_bid(&self) -> &PxQtLadder {
        &self.data.bids
    }
    fn get_ask(&self) -> &PxQtLadder {
        &self.data.asks
    }
}
impl PxQtBook for OrderBookData {
    fn get_bid(&self) -> &PxQtLadder {
        &self.bids
    }
    fn get_ask(&self) -> &PxQtLadder {
        &self.asks
    }
}

// The offset for the order book data in the message where the order book data starts
const BITSTAMP_ORDERBOOK_DATA_OFFSET: usize = 8;
const DATA_PREFIX: &str = "{\"data\":";
pub fn skip_to_orderbook_data(json_str: &str) -> Result<&str, Box<dyn Error>> {
    let json_len = json_str.len();
    if json_len > BITSTAMP_ORDERBOOK_DATA_OFFSET {
        Ok(&json_str[BITSTAMP_ORDERBOOK_DATA_OFFSET..])
    } else {
        Err("Invalid JSON length".into())
    }
}

pub async fn connect_websocket(
    connector: Connector,
    url: &str,
    symbol: &str,
    tx: &PxQtLadderSender,
    top_n: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async_tls_with_config(url, None, true, Some(connector)).await?;
    tracing::info!("Connected to Bitstamp WebSocket API");

    let (mut write, mut read) = ws_stream.split();
    let channel_name = format!("order_book_{}", symbol);
    let subscription =
        Subscription { event: "bts:subscribe".to_string(), data: SubscriptionData { channel: channel_name.clone() } };

    let subscription_json = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_json.into())).await?;
    tracing::info!("Sent subscription request for {}", symbol);

    // Handle incoming messages
    tracing::info!("Waiting for order updates...");
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    if text.starts_with(DATA_PREFIX) {
                        match skip_to_orderbook_data(&text) {
                            Ok(data) => match crate::data_handler::get_top_n_bids_asks_raw(data, top_n) {
                                Ok((bids, asks)) => {
                                    match tx.send((bids, asks)) {
                                        Ok(_) => {
                                            tracing::trace!("Sent order book data to BookWorker");
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to send order book data to BookWorker: {}", e);
                                            break;
                                        }
                                    };
                                }
                                Err(e) => {
                                    tracing::error!("Failed parsing orderbook data in message: {}", e);
                                    tracing::debug!("Raw message: {}", text);
                                }
                            },
                            Err(e) => {
                                tracing::error!("Failed skipping data tag in message: {}", e);
                                tracing::debug!("Raw message: {}", text);
                            }
                        }
                    } else {
                        match serde_json::from_str::<InStreamMessage>(&text) {
                            Ok(parsed) => {
                                // Handle different event types
                                if let Some(event) = &parsed.event {
                                    match event.as_str() {
                                        "bts:subscription_succeeded" => {
                                            tracing::info!("Successfully subscribed to channel: {:?}", parsed.channel);
                                        }
                                        "order_created" | "order_changed" | "order_deleted" => {
                                            tracing::info!("Order event: {}", event);
                                            if let Some(data) = parsed.data {
                                                tracing::info!("Order data: {:#?}", data);
                                            }
                                        }
                                        "bts:error" => {
                                            tracing::error!("Error received: {:?}", parsed.data);
                                            break;
                                        }
                                        "bts:request_reconnect" => {
                                            tracing::warn!("Restart app, bts requested reconnection!!!");
                                            break;
                                        }
                                        "bts:heartbeat" => {
                                            tracing::trace!("TODO: handle the heartbeat.")
                                        }
                                        _ => {
                                            tracing::info!("Received other event: {}", event);
                                            tracing::debug!("Data: {:?}", parsed.data);
                                        }
                                    }
                                } else {
                                    tracing::warn!("Received message without event: {}", text);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse message: {}", e);
                                tracing::debug!("Raw message: {}", text);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error receiving message: {}", e);
                break;
            }
        }
    }

    let unsubscription =
        Subscription { event: "bts:unsubscribe".to_string(), data: SubscriptionData { channel: channel_name.clone() } };

    let unsubscription_json = serde_json::to_string(&unsubscription)?;
    write.send(Message::Text(unsubscription_json.into())).await?;
    tracing::info!("Sent unsubscription request for {}", &channel_name);
    if let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    tracing::debug!("Received unsubscription message: {}", text);
                }
            }
            Err(e) => {
                tracing::error!("Error receiving unsubscription message: {}", e);
            }
        }
    } else {
        tracing::info!("No more messages received.");
    }

    // Disconnect
    write.close().await?;
    tracing::info!("Disconnected from Bitstamp WebSocket API");
    Ok(())
}

pub async fn request_snapshot(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(url).await?.error_for_status();
    Ok(response?.text().await?)
}
