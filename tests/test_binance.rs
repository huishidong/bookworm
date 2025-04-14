use bookworm::tls::get_tls_connector;
use bookworm::binance;
use common::CERT_FILE_PATH;

mod common;

#[tokio::test]
async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    let connector = get_tls_connector(CERT_FILE_PATH)?;
    // wss://stream.binance.com:9443
    // Test failed with error: HTTP error: 451 Unavailable For Legal Reasons
    // wss://stream.testnet.binance.vision
    // Test failed with error: HTTP error: 404 Not Found
    let url = String::from("wss://stream.testnet.binance.vision/ws"); // works

    binance::connect_websocket(connector, &url, "btcusdt").await
}
