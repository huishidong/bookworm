use bookworm::book_worker::Merger;
use bookworm::data_tunnel::*;
use tokio::join;
use tokio::time::{self, Duration};

#[tokio::test]
async fn test_data_tunnel() {
    let (tx_a, rx_a) = create_data_tunnel();
    let (tx_b, rx_b) = create_data_tunnel();
    let worker = Merger::new(10, "Binance".to_string(), rx_a, "Bitstamp".to_string(), rx_b);

    let (_, _, _) = join!(
        tokio::spawn(producer("Binance", tx_a)),
        tokio::spawn(producer("Bitstamp", tx_b)),
        tokio::spawn(worker.run(None))
    );
}

async fn producer(name: &str, tx: PxQtLadderSender) {
    let mut interval = time::interval(Duration::from_millis(500));

    loop {
        interval.tick().await;
        let bid = PxQtLadder::new();
        let ask = PxQtLadder::new();
        if tx.send((bid, ask)).is_err() {
            tracing::info!("Producer {name} channel closed");
            break;
        }
    }
}
