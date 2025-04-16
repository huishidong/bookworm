//! This module, `data_tunnel`, provides utilities for managing data flow within the application
//! through the use of data tunnels. These tunnels facilitate communication between different
//! components of the system by enabling the transmission of structured data types.
//!
//! # Overview
//! The module includes functionality for creating and handling two types of data tunnels:
//!
//! 1. **PxQtLadder Tunnel**: Used for transmitting pairs of `PxQtLadder` data structures.
//! 2. **Summary Tunnel**: Used for transmitting `Summary` objects wrapped in a `Result` type,
//!    which allows for error handling via `tonic::Status`.
//!
//! These tunnels are implemented using Tokio's asynchronous message-passing utilities, such as
//! `mpsc::UnboundedSender` and `mpsc::UnboundedReceiver`. Additionally, the `UnboundedReceiverStream`
//! wrapper is used to integrate with asynchronous streams.
//!
//! # Key Types
//! - `PxQtLadderSender` and `PxQtLadderReceiver`: Types for sending and receiving pairs of
//!   `PxQtLadder` objects.
//! - `SummarySender` and `SummaryReceiver`: Types for sending and receiving `Summary` objects
//!   wrapped in a `Result`.
//! - `SummaryTunnel`: A stream-based abstraction for receiving `Summary` objects asynchronously.
//!
//! # Functions
//! - `create_data_tunnel`: Creates a new `PxQtLadder` data tunnel, returning a sender and receiver
//!   pair.
//! - `create_summary_tunnel`: Creates a new `Summary` data tunnel, returning a sender and receiver
//!   pair.
//!
//! # Usage
//! This module is designed to simplify the process of setting up and managing asynchronous
//! communication channels for structured data. It is particularly useful in scenarios where
//! multiple components need to exchange data in a non-blocking manner.

use super::data_types::Summary;
pub use crate::data_types::PxQtLadder;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::Status;

pub type PxQtLadderSender = mpsc::UnboundedSender<(PxQtLadder, PxQtLadder)>;
pub type PxQtLadderReceiver = mpsc::UnboundedReceiver<(PxQtLadder, PxQtLadder)>;

pub fn create_data_tunnel() -> (PxQtLadderSender, PxQtLadderReceiver) {
    mpsc::unbounded_channel()
}

pub type SummaryTunnel = UnboundedReceiverStream<Result<Summary, Status>>;
pub type SummarySender = mpsc::UnboundedSender<Result<Summary, Status>>;
pub type SummaryReceiver = mpsc::UnboundedReceiver<Result<Summary, Status>>;

pub fn create_summary_tunnel() -> (SummarySender, SummaryReceiver) {
    mpsc::unbounded_channel()
}
