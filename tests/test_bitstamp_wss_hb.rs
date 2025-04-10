use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::protocol::Message};
use bookworm::tls::get_tls_connector;
use bookworm::bitstamp::Heartbeat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = get_tls_connector()?;
    let url = String::from("wss://ws.bitstamp.net");

    let (ws_stream, _) = connect_async_tls_with_config(
        url, None, true, Some(connector)
    ).await?;
    println!("Connected to the web socket");
    let (mut write, mut read) = ws_stream.split();


    // Send a message
    let subscription = Heartbeat {
        event: "bts:heartbeat".to_string()
    };
    let subscription_json = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_json.into())).await?;
    println!("Sent subscription request for heartbeat");

    // Read messages
    let msg = read.next().await;
    match msg {
        Some(Ok(Message::Text(text))) => {
            println!("Received: {}", text);
        }
        Some(Ok(Message::Close(_))) => {
            println!("Connection closed");
        }
        Some(Err(e)) => {
            eprintln!("Error: {}", e);
        }
        _ => {}
    }
    println!("Exiting");
    // Disconnect
    write.close().await.unwrap();
    println!("Disconnected");
    Ok(())
}