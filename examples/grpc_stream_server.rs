use tonic::{transport::Server, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use orderbook::{Summary, Level, Empty};

mod orderbook {
    tonic::include_proto!("orderbook");
}

#[derive(Debug, Default)]
struct MyOrderbookAggregator {}

#[tonic::async_trait]
impl OrderbookAggregator for MyOrderbookAggregator {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for i in 0..5 {
                let summary = Summary {
                    spread: i as f64 * 0.1,
                    bids: vec![
                        Level {
                            exchange: "Binance".to_string(),
                            price: 100.0 + i as f64,
                            amount: 5.0,
                        },
                        Level {
                            exchange: "Bitstamp".to_string(),
                            price: 101.0 + i as f64,
                            amount: 10.0,
                        },
                    ],
                    asks: vec![
                        Level {
                            exchange: "Binance".to_string(),
                            price: 102.0 + i as f64,
                            amount: 5.0,
                        },
                        Level {
                            exchange: "Bitstamp".to_string(),
                            price: 103.0 + i as f64,
                            amount: 10.0,
                        },
                    ],
                };

                if let Err(_) = tx.send(Ok(summary)).await {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "[::1]:50051".parse()?;
    let orderbook_service = MyOrderbookAggregator::default();

    println!("OrderbookAggregator Server listening on {}", address);

    Server::builder()
        .add_service(OrderbookAggregatorServer::new(orderbook_service))
        .serve(address)
        .await?;

    Ok(())
}
