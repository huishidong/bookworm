use tokio::time::{self, Duration};
use bookworm::databus::*;
use bookworm::bookworker::merger;

#[tokio::test]
async fn test_databus() {
    let (tx_a, rx_a) = create_databus();
    let (tx_b, rx_b) = create_databus();

    // Spawn producer tasks
    tokio::spawn(producer("Binance", tx_a));
    tokio::spawn(producer("Bitstamp", tx_b));

    // Spawn merger
    tokio::spawn(merger(rx_a, rx_b));

    // Let app run for a while
    tokio::time::sleep(Duration::from_secs(10)).await;
}

async fn producer(name: &str, tx: PriceLevelSender) {
    let mut interval = time::interval(Duration::from_millis(500));

    loop {
        interval.tick().await;
        let bid = PxQtLadder::new();
        let ask = PxQtLadder::new();
        if tx.send((bid, ask)).is_err() {
            println!("Producer {name} channel closed");
            break;
        }
    }
}


