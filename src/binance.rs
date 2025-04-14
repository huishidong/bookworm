use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::protocol::Message};
use super::price_level::{PxQtLadder, BidAsk};
#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
   pub method: String, // "SUBSCRIBE", "UNSUBSCRIBE"
   pub params: Vec<String>, // ["btcusdt@aggTrade", "btcusdt@depth"]
   pub id: u32, // 1
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
impl BidAsk for OrderBook {
    fn get_bid(&self) -> &PxQtLadder {
        &self.bids
    }
    fn get_ask(&self) -> &PxQtLadder {
        &self.asks
    }    
}

pub async fn connect_websocket(connector: Connector, url: &str, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async_tls_with_config(
        url, None, true, Some(connector)
    ).await?;
    println!("Connected to WebSocket API");
    
    let (mut write, mut read) = ws_stream.split();
    let channel_name = format!("{}@depth10@100ms", symbol);
    let subscription = Subscription {
        method: "SUBSCRIBE".to_string(),
        params: vec![channel_name.clone()],
        id: 1,
    };
    
    let subscription_json = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_json.into())).await?;
    println!("Sent subscription request for {}", &channel_name);

    // Handle incoming messages
    println!("Waiting for order updates...");
    let mut count = 0;
    while let Some(message) = read.next().await {
        if count > 6 {
            break;
        }

        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    match crate::feedhandler::get_top_n_bids_asks_raw(&text, 10) {
                        Ok((bids, asks)) => {
                            if !bids.is_empty() {
                                println!("Top 10 Bids: {:?}", bids);
                            }
                            if !asks.is_empty() {
                                println!("Top 10 Asks: {:?}", asks);
                            }
                            if bids.is_empty() && asks.is_empty() {
                                println!("No bids or asks found, msg: {}", text);
                            }
                        },
                        Err(e) => {
                            println!("Failed to parse message: {}", e);
                            println!("Raw message: {}", text);
                        }
                    }
                } else if let Message::Ping(ping) = msg {
                    println!("Received Ping message: {:?}", ping);
                    write.send(Message::Pong(ping)).await?;
                    count += 1;
                } else if let Message::Close(_) = msg {
                    println!("Connection closed");
                    break;
                } else {
                    println!("Received other message type message[{count}]: {:?}", msg);
                }
            },
            Err(e) => {
                println!("Error receiving message: {}", e);
                break;
            }
        }
    }


    let unsubscription = Subscription {
        method: "UNSUBSCRIBE".to_string(),
        params: vec![channel_name],
        id: 1,
    };
    
    let unsubscription_json = serde_json::to_string(&unsubscription)?;
    write.send(Message::Text(unsubscription_json.into())).await?;
    println!("Sent unsubscription request");
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
