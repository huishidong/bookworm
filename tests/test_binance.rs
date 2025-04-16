use bookworm::binance;
use bookworm::book_worker::Merger;
use bookworm::data_tunnel::create_data_tunnel;
use bookworm::tls::get_tls_connector;
use common::CERT_FILE_PATH;
use tokio::join;

mod common;

#[tokio::test]
async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    let connector = get_tls_connector(CERT_FILE_PATH)?;
    // wss://stream.binance.com:9443
    // Test failed with error: HTTP error: 451 Unavailable For Legal Reasons
    // wss://stream.testnet.binance.vision
    // Test failed with error: HTTP error: 404 Not Found
    let url = String::from("wss://stream.testnet.binance.vision/ws"); // works

    let (tx_a, rx_a) = create_data_tunnel();
    let (_, rx_b) = create_data_tunnel();
    let worker = Merger::new(10, "Binance".to_string(), rx_a, "Bitstamp".to_string(), rx_b);

    let (_, _) = join!(
        binance::connect_websocket(connector, &url, "btcusdt", &tx_a, worker.top_n),
        tokio::spawn(worker.run(None))
    );

    Ok(())
}
