//! This module provides data types and utilities for working with order book data.
//!
//! The module includes:
//! - Protobuf-generated types for representing order book levels and summaries.
//! - Utility functions for creating and comparing price levels.
//! - Traits and type aliases for working with price-quantity ladders.
//!
//! # Types
//! - `ExchangePriceLevel`: Represents a price level on an exchange, including the exchange name, price, and amount.
//! - `Summary`: Represents a summary of the order book, including spread, bids, and asks.
//! - `PxQt`: A type alias for a tuple of price and quantity as strings.
//! - `PxQtLadder`: A type alias for a vector of `PxQt` pairs.
//!
//! # Traits
//! - `PxQtBook`: A trait for accessing bid and ask ladders.
//!
//! # Functions
//! - `make_exchange_price_level`: Creates an `ExchangePriceLevel` from an exchange name and a `PxQt` pair.
//! - `compare_px`: Compares two price strings and returns their ordering.
//!
//! # Examples
//! ```rust
//! use crate::data_types::{make_exchange_price_level, PxQt};
//!
//! let pxqt: PxQt = ["100.0".to_string(), "5.0".to_string()];
//! let level = make_exchange_price_level("Binance".to_string(), &pxqt);
//! assert_eq!(level.exchange, "Binance");
//! assert_eq!(level.price, 100.0);
//! assert_eq!(level.amount, 5.0);
//! ```
//!
//! # Notes
//! - The `orderbook_proto` module is generated from Protobuf definitions and must be available in the build output directory.
//! - The `compare_px` function expects valid numeric strings and will panic if parsing fails.
pub mod orderbook_proto {
    include!(concat!(env!("OUT_DIR"), "/orderbook.rs"));
}

pub use orderbook_proto::Level as ExchangePriceLevel;
pub use orderbook_proto::Summary;

pub type PxQt = [String; 2];
pub fn make_exchange_price_level(exchange: String, pxqt: &PxQt) -> ExchangePriceLevel {
    ExchangePriceLevel { exchange, price: pxqt[0].parse().unwrap_or(0.0), amount: pxqt[1].parse().unwrap_or(0.0) }
}

pub fn compare_px(a: &str, b: &str) -> std::cmp::Ordering {
    let na: f64 = a.parse().expect("invalid number");
    let nb: f64 = b.parse().expect("invalid number");
    na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
}

pub type PxQtLadder = Vec<PxQt>;
pub trait PxQtBook {
    fn get_bid(&self) -> &PxQtLadder;
    fn get_ask(&self) -> &PxQtLadder;
}

impl Eq for ExchangePriceLevel {}

impl Ord for ExchangePriceLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price.partial_cmp(&other.price).unwrap()
    }
}

impl PartialOrd for ExchangePriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::{ExchangePriceLevel, Summary};

    #[test]
    fn test_protobuf_data() {
        let summary = Summary {
            spread: 0.5,
            bids: vec![
                ExchangePriceLevel { exchange: String::from("Binance"), price: 100.0, amount: 5.0 },
                ExchangePriceLevel { exchange: String::from("Coinbase"), price: 101.0, amount: 10.0 },
            ],
            asks: vec![
                ExchangePriceLevel { exchange: String::from("Binance"), price: 102.0, amount: 5.0 },
                ExchangePriceLevel { exchange: String::from("Coinbase"), price: 103.0, amount: 10.0 },
            ],
        };
        println!("Summary: {:?}", summary);
    }
}
