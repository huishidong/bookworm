use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::protocol::Message};

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
pub struct OrderBook {
    pub lastUpdateId: u64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

pub async fn connect_websocket(connector: Connector, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async_tls_with_config(
        url, None, true, Some(connector)
    ).await?;
    println!("Connected to WebSocket API");
    
    let (mut write, mut read) = ws_stream.split();
    
    let subscription = Subscription {
        method: "SUBSCRIBE".to_string(),
        params: vec!["btcusdt@depth10@100ms".to_string()],
        id: 1,
    };
    
    let subscription_json = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_json.into())).await?;
    println!("Sent subscription request for btcusdt@depth10@100ms");

    // Handle incoming messages
    println!("Waiting for order updates...");
    let mut count = 0;
    while let Some(message) = read.next().await {
        count += 1;
        if count > 100  {
            break;
        }

        println!("Received message[{count}]: {:?}", message);
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    println!("Received Text message: {}", text);
                    //// TODO: handle ping/pong and disconnect/connect msgs
                    match serde_json::from_str::<OrderBook>(&text) {
                        Ok(parsed) => {
                            println!("OrderBook data: {:#?}", parsed);
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
        method: "UNSUBSCRIBE".to_string(),
        params: vec!["btcusdt@depth10@100ms".to_string()],
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
