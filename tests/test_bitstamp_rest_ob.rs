use bookworm::bitstamp;
use std::fs;
use std::fs::File;
use std::io::Write;

#[tokio::test]
#[ignore]
async fn test_rest_api() -> Result<(), Box<dyn std::error::Error>> {
    let url = String::from("https://www.bitstamp.net/api/v2/order_book/btcusd?group=1");
    let path = "tests/data/order_book.json";
    match bitstamp::request_snapshot(&url).await {
        Ok(body) => {
            let mut file = File::create(path)?;
            file.write_all(body.as_bytes())?;
            tracing::info!("JSON data saved to '{}'", &path);
        }
        Err(err) => tracing::error!("Error fetching body: {}", err),
    }

    let bookstring = fs::read(path)?;
    let orderbook = serde_json::from_slice::<bitstamp::OrderBook>(&bookstring)?;
    tracing::info!("Order Book: {:?}", orderbook);

    Ok(())
}
