use bookworm::tls::get_tls_connector;
use bookworm::binance::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = get_tls_connector()?;
    // wss://stream.binance.com:9443
    // Test failed with error: HTTP error: 451 Unavailable For Legal Reasons
    // wss://stream.testnet.binance.vision
    // Test failed with error: HTTP error: 404 Not Found
    let url = String::from("wss://stream.testnet.binance.vision/ws");

    if let Err(e) = connect_websocket(connector, &url).await {
        eprintln!("Test failed with error: {}", e);
        panic!("WebSocket test failed");
    }
    Ok(())
}