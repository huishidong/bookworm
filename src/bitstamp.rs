use std::fs;
use std::error::Error;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::protocol::Message};

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
    pub timestamp: String,
    pub microtimestamp: String,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

pub async fn connect_websocket(connector: Connector, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async_tls_with_config(
        url, None, true, Some(connector)
    ).await?;
    println!("Connected to Bitstamp WebSocket API");
    
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to the order book channel for BTC/USD
    // var subscribeMsg = {
    //     "event": "bts:subscribe",
    //     "data": {
    //         "channel": "diff_order_book_btcusd"
    //     }
    // };
    let subscription = Subscription {
        event: "bts:subscribe".to_string(),
        data: SubscriptionData {
            channel: "order_book_btcusd".to_string(),
        },
    };
    
    let subscription_json = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_json.into())).await?;
    println!("Sent subscription request for btcusd");

    // Handle incoming messages
    println!("Waiting for order updates...");
    let mut count = 0;
    while let Some(message) = read.next().await {
        count += 1;
        // match count {
        //     20 => {
        //         println!("Received first message")
        //     },
        //     100 => break
        // }
        if count > 100  {
            break;
        }

        println!("Received message[{count}]: {:?}", message);
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
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
                                    "data" => {
                                        println!("LiveOrderBook event: {}", event);
                                        if let Some(data) = parsed.data {
                                            match serde_json::from_str::<OrderBook>(data.to_string().as_str()) {
                                                Ok(book) => {
                                                    println!("OrderBook data: {:#?}", book);
                                                },
                                                Err(e) => {
                                                    println!("Failed to parse message: {}", e);
                                                    println!("Raw message: {}", text);
                                                }
                                            }
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
            channel: "order_book_btcusd".to_string(),
        },
    };
    
    let unsubscription_json = serde_json::to_string(&unsubscription)?;
    write.send(Message::Text(unsubscription_json.into())).await?;
    println!("Sent unsubscription request for live_orders_btcusd");
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
    println!("Disconnected");
    Ok(())
}

pub async fn request_snapshot(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(url).await?.error_for_status();
    Ok(response?.text().await?)
}
