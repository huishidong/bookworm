use bookworm::tls::get_tls_connector;
use bookworm::bitstamp::connect_websocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = get_tls_connector()?;
    let url = String::from("wss://ws.bitstamp.net");

    if let Err(e) = connect_websocket(connector, &url).await {
        eprintln!("Test failed with error: {}", e);
        panic!("WebSocket test failed");
    }
    Ok(())
}