use bookworm::bitstamp;
use bookworm::book_worker::Merger;
use bookworm::data_tunnel::create_data_tunnel;
use bookworm::tls::get_tls_connector;
use common::CERT_FILE_PATH;
use tokio::join;

mod common;
#[tokio::test]
async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    let connector = get_tls_connector(CERT_FILE_PATH)?;
    let url = String::from("wss://ws.bitstamp.net");
    let (_, rx_a) = create_data_tunnel();
    let (tx_b, rx_b) = create_data_tunnel();
    let worker = Merger::new(10, "Binance".to_string(), rx_a, "Bitstamp".to_string(), rx_b);

    let (_, _) = join!(
        bitstamp::connect_websocket(connector, &url, "btcusd", &tx_b, worker.top_n),
        tokio::spawn(worker.run(None))
    );

    Ok(())
}
