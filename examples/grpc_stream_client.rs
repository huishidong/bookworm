use orderbook::Empty;
use orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(Empty {});

    let mut stream = client.book_summary(request).await?.into_inner();

    tracing::info!("Streaming orderbook summaries:");

    while let Some(summary) = stream.message().await? {
        tracing::info!("Summary: {:?}", summary);
    }

    Ok(())
}
