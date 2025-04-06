//! # Binance WebSocket Module
//!
//! This module provides functionality to interact with the Binance WebSocket API. It includes
//! structures and functions to handle WebSocket connections, subscribe to order book updates,
//! and process incoming messages.
//!
//! ## Key Features
//! - **WebSocket Connection**: Establishes a secure WebSocket connection to the Binance API.
//! - **Subscription Management**: Allows subscribing and unsubscribing to specific data streams
//!   such as order book updates.
//! - **Order Book Handling**: Processes incoming order book data and sends it to a data tunnel
//!   for further processing.
//!
//! ## Structures
//! - [`Subscription`]: Represents a subscription request to the Binance WebSocket API.
//! - [`ResponseMessage`]: Represents a response message from the Binance WebSocket API.
//! - [`OrderBook`]: Represents the order book data structure, implementing the `PxQtBook` trait
//!   for accessing bid and ask ladders.
//!
//! ## Functions
//! - [`connect_websocket`]: Establishes a WebSocket connection, subscribes to a data stream,
//!   processes incoming messages, and handles disconnection.
//!
//! ## Usage
//! To use this module, you need to:
//! 1. Create a WebSocket connector.
//! 2. Call the `connect_websocket` function with the appropriate parameters, including the
//!    WebSocket URL, symbol, and a sender for transmitting processed data.
//!
//! ## Example
//! ```rust
//! use tokio_tungstenite::Connector;
//! use bookworm::binance::connect_websocket;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let connector = Connector::default();
//!     let url = "wss://stream.binance.com:9443/ws";
//!     let symbol = "btcusdt";
//!     let (tx, _rx) = tokio::sync::mpsc::channel(10);
//!     let top_n = 10;
//!
//!     connect_websocket(connector, url, symbol, &tx, top_n).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Dependencies
//! This module relies on the following crates:
//! - `tokio-tungstenite`: For WebSocket connection handling.
//! - `serde`: For serializing and deserializing JSON messages.
//! - `futures-util`: For asynchronous stream and sink utilities.
//! - `tracing`: For structured logging and debugging.
//!
//! ## Error Handling
//! Errors during WebSocket connection, message parsing, or data transmission are logged using
//! the `tracing` crate. The function `connect_websocket` returns a `Result` to indicate success
//! or failure.

use super::data_tunnel::PxQtLadderSender;
use super::data_types::{PxQtBook, PxQtLadder};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::protocol::Message};
use tracing;

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub method: String,      // "SUBSCRIBE", "UNSUBSCRIBE"
    pub params: Vec<String>, // ["btcusdt@aggTrade", "btcusdt@depth"]
    pub id: u32,             // 1
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub result: Option<String>,
    pub id: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct OrderBook {
    pub lastUpdateId: u64,
    pub bids: PxQtLadder,
    pub asks: PxQtLadder,
}
impl PxQtBook for OrderBook {
    fn get_bid(&self) -> &PxQtLadder {
        &self.bids
    }
    fn get_ask(&self) -> &PxQtLadder {
        &self.asks
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
    tracing::info!("Connected to WebSocket API");

    let (mut write, mut read) = ws_stream.split();
    let channel_name = format!("{}@depth10@100ms", symbol); // TODO: config the channel
    let subscription = Subscription { method: "SUBSCRIBE".to_string(), params: vec![channel_name.clone()], id: 1 };

    let subscription_json = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_json.into())).await?;
    tracing::info!("Sent subscription request for {}", &channel_name);

    // Handle incoming messages
    tracing::debug!("Waiting for order updates...");
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    match crate::data_handler::get_top_n_bids_asks_raw(&text, top_n) {
                        Ok((bids, asks)) => {
                            match tx.send((bids, asks)) {
                                Ok(_) => {
                                    tracing::trace!("Sent binance order book data to BookWorker");
                                }
                                Err(e) => {
                                    tracing::error!("Failed to send binance order book data to BookWorker: {}", e);
                                    break;
                                }
                            };
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse message: {}", e);
                            tracing::debug!("Raw message: {}", text);
                        }
                    }
                } else if let Message::Ping(ping) = msg {
                    tracing::trace!("Received Ping message: {:?}", ping);
                    write.send(Message::Pong(ping)).await?;
                } else if let Message::Close(_) = msg {
                    tracing::info!("Connection closed");
                    break;
                } else {
                    tracing::warn!("Received other message type message: {:?}", msg);
                }
            }
            Err(e) => {
                tracing::error!("Error receiving message: {}", e);
                break;
            }
        }
    }

    let unsubscription = Subscription { method: "UNSUBSCRIBE".to_string(), params: vec![channel_name], id: 1 };

    let unsubscription_json = serde_json::to_string(&unsubscription)?;
    write.send(Message::Text(unsubscription_json.into())).await?;
    tracing::info!("Sent unsubscription request");
    if let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    tracing::trace!("Received unsubscription message: {}", text);
                }
            }
            Err(e) => {
                tracing::warn!("Error receiving unsubscription message: {}", e);
            }
        }
    } else {
        tracing::debug!("No more messages received.");
    }

    // Disconnect
    write.close().await?;
    tracing::info!("Disconnected");
    Ok(())
}
