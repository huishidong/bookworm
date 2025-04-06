//! This module implements the gRPC stream publisher for summary data.
//!
//! It provides functionality to create and manage a gRPC stream tunnel
//! for publishing summary data to clients. The `create_summary_tunnel`
//! function is used to establish the data tunnel for streaming summaries.
use super::data_tunnel::create_summary_tunnel;
use super::data_types::Summary;
use super::data_types::orderbook_proto::{Empty, orderbook_aggregator_server::OrderbookAggregator};
use super::summary_manager::SummaryManager;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status};
use tracing;
use uuid;

#[derive(Clone, Default)]
pub struct SummaryPublisher {
    pub manager: SummaryManager,
}

#[tonic::async_trait]
impl OrderbookAggregator for SummaryPublisher {
    type BookSummaryStream = UnboundedReceiverStream<Result<Summary, Status>>;

    async fn book_summary(&self, _request: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (tx, rx) = create_summary_tunnel();

        let client_id = uuid::Uuid::new_v4();
        tracing::info!("serving book summary for {}", &client_id);
        self.manager.add_summary_sender(client_id, tx).await;

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }
}
