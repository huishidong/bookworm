use bookworm::bitstamp::Heartbeat;
use bookworm::tls::get_tls_connector;
use common::CERT_FILE_PATH;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::protocol::Message};

mod common;
#[tokio::test]
async fn test_heartbeat() -> Result<(), Box<dyn std::error::Error>> {
    let connector = get_tls_connector(CERT_FILE_PATH)?;
    let url = String::from("wss://ws.bitstamp.net");

    let (ws_stream, _) = connect_async_tls_with_config(url, None, true, Some(connector)).await?;
    tracing::info!("Connected to the web socket");
    let (mut write, mut read) = ws_stream.split();

    // Send a message
    let subscription = Heartbeat { event: "bts:heartbeat".to_string() };
    let subscription_json = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_json.into())).await?;
    tracing::info!("Sent subscription request for heartbeat");

    // Read messages
    let msg = read.next().await;
    match msg {
        Some(Ok(Message::Text(text))) => {
            tracing::info!("Received: {}", text);
        }
        Some(Ok(Message::Close(_))) => {
            tracing::info!("Connection closed");
        }
        Some(Err(e)) => {
            tracing::info!("Error: {}", e);
        }
        _ => {}
    }
    tracing::info!("Exiting");
    // Disconnect
    write.close().await.unwrap();
    tracing::info!("Disconnected");
    Ok(())
}
