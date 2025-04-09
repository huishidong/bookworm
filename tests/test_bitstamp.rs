use bookworm::tls::get_tls_connector;
use bookworm::bitstamp;
use common::CERT_FILE_PATH;

mod common;
#[tokio::test]
async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    let connector = get_tls_connector(CERT_FILE_PATH)?;
    let url = String::from("wss://ws.bitstamp.net");
    bitstamp::connect_websocket(connector, &url, "btcusd").await
}
