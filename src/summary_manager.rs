//! # Summary Manager Module
//!
//! This module provides the `SummaryManager` struct, which is responsible for managing
//! and broadcasting summaries to multiple clients. It uses asynchronous communication
//! channels to send summaries and ensures thread-safe access to the underlying data
//! structures through the use of `tokio::sync::RwLock`.
//!
//! ## Key Components
//!
//! - **SummaryManager**: The main struct that manages client connections and broadcasts summaries.
//! - **SummaryTx**: A type alias for an unbounded sender channel used to send `Summary` objects to clients.
//! - **ClientId**: A type alias for `uuid::Uuid`, representing the unique identifier for each client.
//!
//! ## Features
//!
//! - **Add Summary Sender**: Allows adding a new client and its associated sender channel.
//! - **Remove Summary Sender**: Removes a client and its sender channel from the manager.
//! - **Broadcast Summaries**: Sends a summary to all connected clients. If a client fails to receive
//!   the summary, it is automatically removed from the manager.
//!
//! ## Usage
//!
//! This module is designed to be used in asynchronous contexts, leveraging `tokio` for concurrency.
//! It is particularly useful in scenarios where multiple clients need to receive updates or notifications
//! in real-time.
//!
//! ## Example
//!
//! ```rust
//! use crate::summary_manager::SummaryManager;
//! use crate::data_types::Summary;
//! use tokio::sync::mpsc;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!     let manager = SummaryManager::default();
//!     let (tx, _rx) = mpsc::unbounded_channel();
//!     let client_id = Uuid::new_v4();
//!
//!     manager.add_summary_sender(client_id, tx).await;
//!     let summary = Summary { /* fields */ };
//!     manager.broadcast(summary).await;
//!     manager.remove_summary_sender(&client_id).await;
//! }
//! ```
use crate::data_types::Summary;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use tracing;

type SummaryTx = UnboundedSender<Result<Summary, tonic::Status>>;
type ClientId = uuid::Uuid;

#[derive(Clone, Default)]
pub struct SummaryManager {
    inner: Arc<RwLock<HashMap<ClientId, SummaryTx>>>,
}

impl SummaryManager {
    pub async fn add_summary_sender(&self, client_id: ClientId, tx: SummaryTx) {
        self.inner.write().await.insert(client_id, tx);
    }

    pub async fn remove_summary_sender(&self, client_id: &ClientId) {
        self.inner.write().await.remove(client_id);
    }

    pub async fn broadcast(&self, summary: Summary) {
        let mut failed_clients = Vec::new();
        for (client_id, tx) in self.inner.read().await.iter() {
            tracing::trace!("Client {} served", client_id);
            if tx.send(Ok(summary.clone())).is_err() {
                failed_clients.push(*client_id);
            }
        }
        if !failed_clients.is_empty() {
            let mut clients = self.inner.write().await;
            for client_id in failed_clients {
                clients.remove(&client_id);
                tracing::info!("Client {} removed.", client_id);
            }
        }
    }
}
