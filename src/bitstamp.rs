use std::error::Error;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::protocol::Message};
use super::price_level::{PxQtLadder, BidAsk};

#[derive(Debug, Serialize, Deserialize)]
pub struct Heartbeat {
    pub event: String
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
impl BidAsk for OrderBook {
    fn get_bid(&self) -> &PxQtLadder {
        &self.data.bids
    }
    fn get_ask(&self) -> &PxQtLadder {
        &self.data.asks
    }    
}
impl BidAsk for OrderBookData {
    fn get_bid(&self) -> &PxQtLadder {
        &self.bids
    }
    fn get_ask(&self) -> &PxQtLadder {
        &self.asks
    }    
}

/// The offset for the order book data in the message where the order book data starts
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

pub async fn connect_websocket(connector: Connector, url: &str, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async_tls_with_config(
        url, None, true, Some(connector)
    ).await?;
    println!("Connected to Bitstamp WebSocket API");
    
    let (mut write, mut read) = ws_stream.split();
    let channel_name = format!("order_book_{}", symbol);
    let subscription = Subscription {
        event: "bts:subscribe".to_string(),
        data: SubscriptionData {
            channel: channel_name.clone(),
        },
    };
    
    let subscription_json = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_json.into())).await?;
    println!("Sent subscription request for {}", symbol);

    // Handle incoming messages
    println!("Waiting for order updates...");
    let mut count = 0;
    while let Some(message) = read.next().await {
        count += 1;
        if count > 100  {
            break;
        }

        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    if text.starts_with(DATA_PREFIX) {
                        match skip_to_orderbook_data(&text) {
                            Ok(data) => {
                                match crate::feedhandler::get_top_n_bids_asks_raw(data, 10) {
                                    Ok((bids, asks)) => {
                                        println!("Top 10 Bids: {:?}", bids);
                                        println!("Top 10 Asks: {:?}", asks);
                                    },
                                    Err(e) => {
                                        println!("Failed to parse orderbook data in message: {}", e);
                                        println!("Raw message: {}", text);
                                    }
                                }
                            },
                            Err(e) => {
                                println!("Failed to skip data tag in message: {}", e);
                                println!("Raw message: {}", text);
                            }
                        }
                    } else {
                        match serde_json::from_str::<InStreamMessage>(&text) {
                            Ok(parsed) => {
                                // Handle different event types
                                if let Some(event) = &parsed.event {
                                    match event.as_str() {
                                        "bts:subscription_succeeded" => {
                                            println!("Successfully subscribed to channel: {:?}", parsed.channel);
                                        },
                                        "order_created" | "order_changed" | "order_deleted" => {
                                            println!("Order event: {}", event);
                                            if let Some(data) = parsed.data {
                                                println!("Order data: {:#?}", data);
                                            }
                                        },
                                        "bts:error" => {
                                            println!("Error received: {:?}", parsed.data);
                                        },
                                        _ => {
                                            println!("Received other event: {}", event);
                                            println!("Data: {:?}", parsed.data);
                                        }
                                    }
                                    //// TODO: handle heartbeat and disconnect/connect msgs
                                } else {
                                    println!("Received message without event: {}", text);
                                }
                            },
                            Err(e) => {
                                println!("Failed to parse message: {}", e);
                                println!("Raw message: {}", text);
                            }
                        }
                    }
                }
            },
            Err(e) => {
                println!("Error receiving message: {}", e);
                break;
            }
        }
    }


    let unsubscription = Subscription {
        event: "bts:unsubscribe".to_string(),
        data: SubscriptionData {
            channel: channel_name.clone(),
        },
    };
    
    let unsubscription_json = serde_json::to_string(&unsubscription)?;
    write.send(Message::Text(unsubscription_json.into())).await?;
    println!("Sent unsubscription request for {}", &channel_name);
    if let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    println!("Received unsubscription message: {}", text);
                }
            },
            Err(e) => {
                println!("Error receiving unsubscription message: {}", e);
            }
        }
    } else {
        println!("No more messages received.");
    }

    // Disconnect
    write.close().await?;
    println!("Disconnected from Bitstamp WebSocket API");
    Ok(())
}

pub async fn request_snapshot(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(url).await?.error_for_status();
    Ok(response?.text().await?)
}
