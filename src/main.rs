use bookworm::binance;
use bookworm::bitstamp;
use bookworm::book_worker::Merger;
use bookworm::data_tunnel::*;
use bookworm::data_types::orderbook_proto::orderbook_aggregator_server::OrderbookAggregatorServer;
use bookworm::summary_publisher::SummaryPublisher;
use bookworm::tls::get_tls_connector;
use tokio::join;
use tonic::transport::Server;

const CERT_FILE_PATH: &str = "/home/huishi/work/service/certs/localhost.crt";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let address = "[::1]:50051".parse().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    let (top_n, exchange_a, exchange_b) = (5, "Binance", "Bitstamp");
    let summary_publisher = SummaryPublisher::default();
    tracing::info!("SummaryPublisher Server listening on {}", address);

    let (binance_url, bitstamp_url) = ("wss://stream.testnet.binance.vision/ws", "wss://ws.bitstamp.net");
    let (tx_a, rx_a) = create_data_tunnel();
    let (tx_b, rx_b) = create_data_tunnel();

    let worker = Merger::new(top_n, exchange_a.to_string(), rx_a, exchange_b.to_string(), rx_b);
    let (_, _, _, _) = join!(
        Server::builder().add_service(OrderbookAggregatorServer::new(summary_publisher.clone())).serve(address),
        binance::connect_websocket(get_tls_connector(CERT_FILE_PATH).unwrap(), binance_url, "btcusdt", &tx_a, top_n),
        bitstamp::connect_websocket(get_tls_connector(CERT_FILE_PATH).unwrap(), bitstamp_url, "btcusd", &tx_b, top_n),
        tokio::spawn(worker.run(Some(summary_publisher.manager.clone()))),
    );

    Ok(())
}
