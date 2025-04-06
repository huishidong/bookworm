//! This module defines the functionality for working with books in the application.
//!
//! It utilizes components from the `data_tunnel` module, such as `PxQtLadder` and
//! `PxQtLadderReceiver`, to facilitate communication and data processing related to books.
//!
//! The primary purpose of this module is to handle book-related operations, ensuring
//! efficient data flow and processing within the application.
//!
//! The supported worker include:
//! - **`Merger`**: merge two PxQx Ladders and publish the summary via a summary manager
//!
use crate::data_tunnel::{PxQtLadder, PxQtLadderReceiver};
use crate::data_types::{ExchangePriceLevel, compare_px, make_exchange_price_level};
use crate::summary_manager::SummaryManager;
use tracing;

#[derive(Debug)]
pub struct Merger {
    pub top_n: usize,
    pub book_name_a: String,
    pub book_name_b: String,
    rx_a: PxQtLadderReceiver,
    rx_b: PxQtLadderReceiver,
}

impl Merger {
    pub fn new(
        top_n: usize,
        name_a: String,
        rx_a: PxQtLadderReceiver,
        name_b: String,
        rx_b: PxQtLadderReceiver,
    ) -> Self {
        Merger { top_n, book_name_a: name_a, book_name_b: name_b, rx_a, rx_b }
    }

    pub async fn run(self, tx: Option<SummaryManager>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.merge(tx).await
    }

    fn merge_bids(
        &mut self,
        from_exchange: &str,
        from_level: &PxQtLadder,
        to_exchange: &str,
        to_level: &PxQtLadder,
        merged_bids: &mut Vec<ExchangePriceLevel>,
    ) {
        merged_bids.clear();
        let (mut i, mut j, mut k) = (0, 0, 0);
        let (n_i, n_j) = (from_level.len(), to_level.len());
        while i < n_i && j < n_j && k < self.top_n {
            match compare_px(&from_level[i][0], &to_level[j][0]) {
                std::cmp::Ordering::Greater => {
                    merged_bids.push(make_exchange_price_level(from_exchange.to_string(), &from_level[i]));
                    i += 1;
                    k += 1;
                }
                std::cmp::Ordering::Less => {
                    merged_bids.push(make_exchange_price_level(to_exchange.to_string(), &to_level[j]));
                    j += 1;
                    k += 1;
                }
                std::cmp::Ordering::Equal => {
                    merged_bids.push(make_exchange_price_level(from_exchange.to_string(), &from_level[i]));
                    i += 1;
                    k += 1;
                    if k < self.top_n {
                        merged_bids.push(make_exchange_price_level(to_exchange.to_string(), &to_level[j]));
                        j += 1;
                        k += 1;
                    }
                }
            }
        }
        while i < n_i && k < self.top_n {
            merged_bids.push(make_exchange_price_level(from_exchange.to_string(), &from_level[i]));
            i += 1;
            k += 1;
        }
        while j < n_j && k < self.top_n {
            merged_bids.push(make_exchange_price_level(to_exchange.to_string(), &to_level[j]));
            j += 1;
            k += 1;
        }
    }

    fn merge_asks(
        &mut self,
        from_exchange: &str,
        from_level: &PxQtLadder,
        to_exchange: &str,
        to_level: &PxQtLadder,
        merged_asks: &mut Vec<ExchangePriceLevel>,
    ) {
        merged_asks.clear();
        let (mut i, mut j, mut k) = (0, 0, 0);
        let (n_i, n_j) = (from_level.len(), to_level.len());
        while i < n_i && j < n_j && k < self.top_n {
            match compare_px(&from_level[i][0], &to_level[j][0]) {
                std::cmp::Ordering::Less => {
                    merged_asks.push(make_exchange_price_level(from_exchange.to_string(), &from_level[i]));
                    i += 1;
                    k += 1;
                }
                std::cmp::Ordering::Greater => {
                    merged_asks.push(make_exchange_price_level(to_exchange.to_string(), &to_level[j]));
                    j += 1;
                    k += 1;
                }
                std::cmp::Ordering::Equal => {
                    merged_asks.push(make_exchange_price_level(from_exchange.to_string(), &from_level[i]));
                    i += 1;
                    k += 1;
                    if k < self.top_n {
                        merged_asks.push(make_exchange_price_level(to_exchange.to_string(), &to_level[j]));
                        j += 1;
                        k += 1;
                    }
                }
            }
        }
        while i < n_i && k < self.top_n {
            merged_asks.push(make_exchange_price_level(from_exchange.to_string(), &from_level[i]));
            i += 1;
            k += 1;
        }
        while j < n_j && k < self.top_n {
            merged_asks.push(make_exchange_price_level(to_exchange.to_string(), &to_level[j]));
            j += 1;
            k += 1;
        }
    }

    async fn merge(mut self, tx: Option<SummaryManager>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut merged_bids = Vec::<ExchangePriceLevel>::with_capacity(self.top_n);
        let mut merged_asks = Vec::<ExchangePriceLevel>::with_capacity(self.top_n);
        let mut last_bids_a = PxQtLadder::new();
        let mut last_asks_a = PxQtLadder::new();
        let mut last_bids_b = PxQtLadder::new();
        let mut last_asks_b = PxQtLadder::new();

        loop {
            tokio::select! {
                Some((bids_a, asks_a)) = self.rx_a.recv() => {
                    let from_exchange = self.book_name_a.to_string();
                    let to_exchange = self.book_name_b.to_string();
                    tracing::trace!("Received from {}: \n {:?} \n {:?}", &from_exchange, &bids_a, &asks_a);
                    last_bids_a = bids_a;
                    last_asks_a = asks_a;
                    self.merge_bids(
                        &from_exchange,
                        &last_bids_a,
                        &to_exchange,
                        &last_bids_b,
                        &mut merged_bids
                    );
                    self.merge_asks(
                        &from_exchange,
                        &last_asks_a,
                        &to_exchange,
                        &last_asks_b,
                        &mut merged_asks
                    );
                }
                Some((bids_b, asks_b)) = self.rx_b.recv() => {
                    let from_exchange = self.book_name_b.to_string();
                    let to_exchange = self.book_name_a.to_string();
                    tracing::trace!("Received from {}: \n {:?}, \n {:?}", &from_exchange, &bids_b, &asks_b);
                    last_bids_b = bids_b;
                    last_asks_b = asks_b;
                    self.merge_bids(
                        &from_exchange,
                        &last_bids_b,
                        &to_exchange,
                        &last_bids_a,
                        &mut merged_bids
                    );
                    self.merge_asks(
                        &from_exchange,
                        &last_asks_b,
                        &to_exchange,
                        &last_asks_a,
                        &mut merged_asks
                    );
                }
                else => {
                    tracing::warn!("Received sth else?");
                },
            }
            tracing::trace!("Merged Bids and Asks: \n {:?}, \n {:?}", merged_bids, merged_asks);
            if let Some(ref tx) = tx {
                let summary = crate::data_types::Summary {
                    spread: if merged_asks.is_empty() || merged_bids.is_empty() {
                        0.0 // or f64::NAN, or any other default value that makes sense for your use case
                    } else {
                        merged_asks[0].price - merged_bids[0].price
                    },
                    bids: merged_bids.clone(),
                    asks: merged_asks.clone(),
                };
                tx.broadcast(summary).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_tunnel::create_data_tunnel;

    fn create_price_ladder(prices: &[[&str; 2]]) -> PxQtLadder {
        prices.iter().map(|price_amount| [price_amount[0].to_string(), price_amount[1].to_string()]).collect()
    }

    fn create_test_worker() -> (Merger, Vec<ExchangePriceLevel>) {
        let (_, rx_a) = create_data_tunnel();
        let (_, rx_b) = create_data_tunnel();
        let worker = Merger::new(3, "BinanceT".to_string(), rx_a, "BitstampT".to_string(), rx_b);
        let merged = Vec::new();
        (worker, merged)
    }
    #[test]
    fn test_merge_bids_with_different_prices() {
        let (mut worker, mut merged) = create_test_worker();

        let binance_bids = create_price_ladder(&[["100.0", "1.0"], ["98.0", "2.0"], ["96.0", "3.0"]]);
        let bitstamp_bids = create_price_ladder(&[["101.0", "4.0"], ["99.0", "5.0"], ["94.0", "6.0"]]);

        worker.merge_bids("BinanceT", &binance_bids, "BitstampT", &bitstamp_bids, &mut merged);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].exchange, "BitstampT");
        assert_eq!(merged[0].price, 101.0);
        assert_eq!(merged[0].amount, 4.0);
        assert_eq!(merged[1].exchange, "BinanceT");
        assert_eq!(merged[1].price, 100.0);
        assert_eq!(merged[1].amount, 1.0);
        assert_eq!(merged[2].exchange, "BitstampT");
        assert_eq!(merged[2].price, 99.0);
        assert_eq!(merged[2].amount, 5.0);
    }

    #[test]
    fn test_merge_bids_with_same_prices() {
        let (mut worker, mut merged) = create_test_worker();
        let binance_bids = create_price_ladder(&[["100.0", "1.0"], ["98.0", "2.0"], ["96.0", "3.0"]]);
        let bitstamp_bids = create_price_ladder(&[["100.0", "4.0"], ["99.0", "5.0"], ["94.0", "6.0"]]);

        worker.merge_bids("BinanceT", &binance_bids, "BitstampT", &bitstamp_bids, &mut merged);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].exchange, "BinanceT");
        assert_eq!(merged[0].price, 100.0);
        assert_eq!(merged[0].amount, 1.0);
        assert_eq!(merged[1].exchange, "BitstampT");
        assert_eq!(merged[1].price, 100.0);
        assert_eq!(merged[1].amount, 4.0);
        assert_eq!(merged[2].exchange, "BitstampT");
        assert_eq!(merged[2].price, 99.0);
        assert_eq!(merged[2].amount, 5.0);
    }

    #[test]
    fn test_merge_asks_with_different_prices() {
        let (mut worker, mut merged) = create_test_worker();
        let binance_asks = create_price_ladder(&[["96.0", "3.0"], ["98.0", "2.0"], ["100.0", "1.0"]]);
        let bitstamp_asks = create_price_ladder(&[["94.0", "6.0"], ["99.0", "5.0"], ["101.0", "4.0"]]);

        worker.merge_asks("BinanceT", &binance_asks, "BitstampT", &bitstamp_asks, &mut merged);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].exchange, "BitstampT");
        assert_eq!(merged[0].price, 94.0);
        assert_eq!(merged[0].amount, 6.0);
        assert_eq!(merged[1].exchange, "BinanceT");
        assert_eq!(merged[1].price, 96.0);
        assert_eq!(merged[1].amount, 3.0);
        assert_eq!(merged[2].exchange, "BinanceT");
        assert_eq!(merged[2].price, 98.0);
        assert_eq!(merged[2].amount, 2.0);
    }

    #[test]
    fn test_merge_asks_with_same_prices() {
        let (mut worker, mut merged) = create_test_worker();
        let binance_asks = create_price_ladder(&[["96.0", "3.0"], ["98.0", "2.0"], ["100.0", "1.0"]]);
        let bitstamp_asks = create_price_ladder(&[["96.0", "6.0"], ["99.0", "5.0"], ["101.0", "4.0"]]);

        worker.merge_asks("BinanceT", &binance_asks, "BitstampT", &bitstamp_asks, &mut merged);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].exchange, "BinanceT");
        assert_eq!(merged[0].price, 96.0);
        assert_eq!(merged[0].amount, 3.0);
        assert_eq!(merged[1].exchange, "BitstampT");
        assert_eq!(merged[1].price, 96.0);
        assert_eq!(merged[1].amount, 6.0);
        assert_eq!(merged[2].exchange, "BinanceT");
        assert_eq!(merged[2].price, 98.0);
        assert_eq!(merged[2].amount, 2.0);
    }

    #[test]
    fn test_merge_empty_onto_asks() {
        let (mut worker, mut merged) = create_test_worker();
        let binance_asks = create_price_ladder(&[]);
        let bitstamp_asks = create_price_ladder(&[["96.0", "6.0"], ["99.0", "5.0"], ["101.0", "4.0"]]);

        worker.merge_asks("BinanceT", &binance_asks, "BitstampT", &bitstamp_asks, &mut merged);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].exchange, "BitstampT");
        assert_eq!(merged[0].price, 96.0);
        assert_eq!(merged[0].amount, 6.0);
        assert_eq!(merged[1].exchange, "BitstampT");
        assert_eq!(merged[1].price, 99.0);
        assert_eq!(merged[1].amount, 5.0);
        assert_eq!(merged[2].exchange, "BitstampT");
        assert_eq!(merged[2].price, 101.0);
        assert_eq!(merged[2].amount, 4.0);
    }

    #[test]
    fn test_merge_asks_with_empty_book() {
        let (mut worker, mut merged) = create_test_worker();
        let binance_asks = create_price_ladder(&[]);
        let bitstamp_asks = create_price_ladder(&[["96.0", "6.0"], ["99.0", "5.0"], ["101.0", "4.0"]]);

        worker.merge_asks("BitstampT", &bitstamp_asks, "BinanceT", &binance_asks, &mut merged);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].exchange, "BitstampT");
        assert_eq!(merged[0].price, 96.0);
        assert_eq!(merged[0].amount, 6.0);
        assert_eq!(merged[1].exchange, "BitstampT");
        assert_eq!(merged[1].price, 99.0);
        assert_eq!(merged[1].amount, 5.0);
        assert_eq!(merged[2].exchange, "BitstampT");
        assert_eq!(merged[2].price, 101.0);
        assert_eq!(merged[2].amount, 4.0);
    }

    #[test]
    fn test_merge_empty_onto_bids() {
        let (mut worker, mut merged) = create_test_worker();
        let bitstamp_bids = create_price_ladder(&[]);
        let binance_bids = create_price_ladder(&[["100.0", "1.0"], ["98.0", "2.0"], ["96.0", "3.0"]]);

        worker.merge_asks("BitstampT", &bitstamp_bids, "BinanceT", &binance_bids, &mut merged);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].exchange, "BinanceT");
        assert_eq!(merged[0].price, 100.0);
        assert_eq!(merged[0].amount, 1.0);
        assert_eq!(merged[1].exchange, "BinanceT");
        assert_eq!(merged[1].price, 98.0);
        assert_eq!(merged[1].amount, 2.0);
        assert_eq!(merged[2].exchange, "BinanceT");
        assert_eq!(merged[2].price, 96.0);
        assert_eq!(merged[2].amount, 3.0);
    }

    #[test]
    fn test_merge_bids_with_empty_book() {
        let (mut worker, mut merged) = create_test_worker();
        let binance_asks = create_price_ladder(&[]);
        let bitstamp_asks = create_price_ladder(&[["100.0", "1.0"], ["98.0", "2.0"], ["96.0", "3.0"]]);

        worker.merge_asks("BitstampT", &bitstamp_asks, "BinanceT", &binance_asks, &mut merged);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].exchange, "BitstampT");
        assert_eq!(merged[0].price, 100.0);
        assert_eq!(merged[0].amount, 1.0);
        assert_eq!(merged[1].exchange, "BitstampT");
        assert_eq!(merged[1].price, 98.0);
        assert_eq!(merged[1].amount, 2.0);
        assert_eq!(merged[2].exchange, "BitstampT");
        assert_eq!(merged[2].price, 96.0);
        assert_eq!(merged[2].amount, 3.0);
    }

    #[test]
    fn test_merge_with_empty_vectors() {
        let (mut worker, _) = create_test_worker();
        let mut merged_bids = Vec::new();
        let mut merged_asks = Vec::new();
        let empty_ladder = create_price_ladder(&[]);
        worker.merge_bids("BinanceT", &empty_ladder, "BitstampT", &empty_ladder, &mut merged_bids);
        worker.merge_asks("BinanceT", &empty_ladder, "BitstampT", &empty_ladder, &mut merged_asks);

        let summary = crate::data_types::Summary {
            spread: if merged_asks.is_empty() || merged_bids.is_empty() {
                0.0
            } else {
                merged_asks[0].price - merged_bids[0].price
            },
            bids: merged_bids.clone(),
            asks: merged_asks.clone(),
        };

        assert_eq!(summary.spread, 0.0);
        assert!(summary.bids.is_empty());
        assert!(summary.asks.is_empty());
    }
}
