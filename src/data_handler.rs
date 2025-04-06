//! # Data Handler Module
//!
//! This module provides functionality for processing and extracting data related to order books,
//! specifically focusing on retrieving the top N bids and asks from JSON data. It is designed to
//! handle different formats of order book data and provides utilities for both structured and raw JSON inputs.
//!
//! ## Features
//!
//! - **`get_top_n`**: A generic function that extracts the top N bids and asks from a JSON string
//!   representing an order book. This function requires the order book type to implement the `PxQtBook`
//!   trait and be deserializable. A complete parsing of the json is done within the function.
//!
//! - **`get_top_n_bids_asks_raw`**: A function that extracts the top N bids and asks directly from raw JSON
//!   data without requiring a specific order book type. This is achieved using custom deserialization logic
//!   to limit the number of elements processed. This function avoid the full parsing, it just traverse the
//!   `bids`` and the `asks`, parsing only the top n [String; 2] of each.
//!
//! ## Dependencies
//!
//! - **`serde`**: Used for JSON deserialization.
//! - **`serde_json`**: Provides JSON parsing and deserialization capabilities.
//!
//! ## Error Handling
//!
//! Both functions return a `Result` type, with the `Ok` variant containing a tuple of `PxQtLadder`
//! for bids and asks, and the `Err` variant containing a boxed error for handling deserialization
//! or parsing issues.
//!
//! ## Testing
//!
//! The module includes comprehensive unit tests to ensure correctness and robustness. The tests cover:
//! - Handling cases where the requested number of levels exceeds the available levels in the JSON data.
//! - Validating the correctness of the extracted top N bids and asks for different order book formats.
//! - Ensuring compatibility with both structured and raw JSON inputs.
//!
//! ## Example Usage
//!
//! ```rust
//! use crate::data_handler::{get_top_n, get_top_n_bids_asks_raw};
//!
//! let json_str = r#"{
//!     "bids": [["100.00", "1"], ["99.00", "2"]],
//!     "asks": [["101.00", "1"], ["102.00", "2"]]
//! }"#;
//!
//! let n = 1;
//! let (top_bids, top_asks) = get_top_n_bids_asks_raw(json_str, n).unwrap();
//!
//! assert_eq!(top_bids.len(), 1);
//! assert_eq!(top_asks.len(), 1);
//! assert_eq!(top_bids[0][0], "100.00");
//! assert_eq!(top_asks[0][0], "101.00");
//! ```
//!
use crate::data_types::{PxQtBook, PxQtLadder};
use serde::de::SeqAccess;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use std::error::Error;
use std::fmt;

pub fn get_top_n<T>(json_str: &str, n: usize) -> Result<(PxQtLadder, PxQtLadder), Box<dyn std::error::Error>>
where
    T: PxQtBook + serde::de::DeserializeOwned,
{
    let orderbook: T = serde_json::from_str(json_str)?;

    let top_bids = orderbook.get_bid().iter().take(n).cloned().collect();
    let top_asks = orderbook.get_ask().iter().take(n).cloned().collect();

    Ok((top_bids, top_asks))
}

pub fn get_top_n_bids_asks_raw(raw_json: &str, n: usize) -> Result<(PxQtLadder, PxQtLadder), Box<dyn Error>> {
    struct TopNArray {
        n: usize,
    }

    impl<'de> de::DeserializeSeed<'de> for TopNArray {
        type Value = PxQtLadder;

        fn deserialize<D>(self, deserializer: D) -> Result<PxQtLadder, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct LimitSeqVisitor {
                n: usize,
                out: PxQtLadder,
            }

            impl<'de> Visitor<'de> for LimitSeqVisitor {
                type Value = PxQtLadder;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("an array of [String; 2]")
                }

                fn visit_seq<A>(mut self, mut seq: A) -> Result<PxQtLadder, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    while self.out.len() < self.n {
                        match seq.next_element()? {
                            Some(item) => self.out.push(item),
                            None => break,
                        }
                    }

                    // Consume the rest silently, ie, skip, for serde_json to not panic when reading further tags.
                    while seq.next_element::<de::IgnoredAny>()?.is_some() {}

                    Ok(self.out)
                }
            }

            deserializer.deserialize_seq(LimitSeqVisitor { n: self.n, out: Vec::with_capacity(self.n) })
        }
    }

    struct TopNVisitor {
        n: usize,
    }

    impl<'de> Visitor<'de> for TopNVisitor {
        type Value = (PxQtLadder, PxQtLadder);

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON object with 'bids' and 'asks'")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut bids = PxQtLadder::new();
            let mut asks = PxQtLadder::new();

            while let Some(key) = map.next_key::<&str>()? {
                match key {
                    "bids" => {
                        bids = map.next_value_seed(TopNArray { n: self.n })?;
                    }
                    "asks" => {
                        asks = map.next_value_seed(TopNArray { n: self.n })?;
                    }
                    _ => {
                        let _ignored: serde_json::Value = map.next_value()?;
                    }
                }
            }

            Ok((bids, asks))
        }
    }

    let visitor = TopNVisitor { n };

    let mut de = serde_json::Deserializer::from_str(raw_json);

    let result = de.deserialize_map(visitor)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bitstamp::skip_to_orderbook_data, test_data::TEST_DATA};

    #[test]
    fn test_binance_get_top_n_not_enough_levels() {
        let n_requested: usize = TEST_DATA.n_level_in_json_str + 1;

        let (top_bids, top_asks) = get_top_n::<crate::binance::OrderBook>(TEST_DATA.json_str, n_requested)
            .expect("Failed to get top n bids and asks");

        assert_eq!(top_bids.len(), TEST_DATA.n_level_in_json_str);
        assert_eq!(top_asks.len(), TEST_DATA.n_level_in_json_str);
        assert_eq!(top_bids[0][0], "100.00");
        assert_eq!(top_bids[0][1], "1");
        assert_eq!(top_asks[0][0], "101.00");
        assert_eq!(top_asks[0][1], "1");
    }

    #[test]
    fn test_bitstamp_get_top_n_not_enough_levels() {
        let n_requested: usize = TEST_DATA.n_level_in_json_str + 1;

        let (top_bids, top_asks) = get_top_n::<crate::bitstamp::OrderBookData>(TEST_DATA.json_str, n_requested)
            .expect("Failed to get top n bids and asks");

        assert_eq!(top_bids.len(), TEST_DATA.n_level_in_json_str);
        assert_eq!(top_asks.len(), TEST_DATA.n_level_in_json_str);
        assert_eq!(top_bids[0][0], "100.00");
        assert_eq!(top_bids[0][1], "1");
        assert_eq!(top_asks[0][0], "101.00");
        assert_eq!(top_asks[0][1], "1");
    }

    #[test]
    fn test_get_top_n_bids_asks_raw_not_enough_levels() {
        let n_requested: usize = TEST_DATA.n_level_in_json_str + 1;

        let (top_bids, top_asks) =
            get_top_n_bids_asks_raw(TEST_DATA.json_str, n_requested).expect("Failed to get top n bids and asks");

        assert_eq!(top_bids.len(), TEST_DATA.n_level_in_json_str);
        assert_eq!(top_asks.len(), TEST_DATA.n_level_in_json_str);
        assert_eq!(top_bids[0][0], "100.00");
        assert_eq!(top_bids[0][1], "1");
        assert_eq!(top_asks[0][0], "101.00");
        assert_eq!(top_asks[0][1], "1");
    }

    #[test]
    fn test_binance_get_top_n() {
        let n: usize = 10;
        let (top_bids, top_asks) = get_top_n::<crate::binance::OrderBook>(TEST_DATA.binance_json_byte, n)
            .expect("Failed to get top n bids and asks");
        println!("top_bids: {:?}", &top_bids);
        println!("top_asks: {:?}", &top_asks);
        assert_eq!(top_bids.len(), n);
        assert_eq!(top_asks.len(), n);
    }

    #[test]
    fn test_bitstamp_get_top_n() {
        let n: usize = 10;
        let (top_bids, top_asks) = get_top_n::<crate::bitstamp::OrderBook>(TEST_DATA.bitstamp_json_byte, n)
            .expect("Failed to get top n bids and asks");
        println!("top_bids: {:?}", &top_bids);
        println!("top_asks: {:?}", &top_asks);
        assert_eq!(top_bids.len(), n);
        assert_eq!(top_asks.len(), n);
    }

    #[test]
    fn test_binance_get_top_n_bids_asks_raw() {
        let n = 5;
        let (top_bids, top_asks) =
            get_top_n_bids_asks_raw(TEST_DATA.binance_json_byte, n).expect("Failed to get top n bids and asks");

        println!("top_bids: {:?}", &top_bids);
        println!("top_asks: {:?}", &top_asks);
        assert_eq!(top_bids.len(), n);
        assert_eq!(top_asks.len(), n);
    }

    #[test]
    fn test_bitstamp_get_top_n_bids_asks_raw() {
        let n = 5;
        let (top_bids, top_asks) =
            get_top_n_bids_asks_raw(skip_to_orderbook_data(TEST_DATA.bitstamp_json_byte).unwrap(), n)
                .expect("Failed to get top n bids and asks");

        println!("top_bids: {:?}", &top_bids);
        println!("top_asks: {:?}", &top_asks);
        assert_eq!(top_bids.len(), n);
        assert_eq!(top_asks.len(), n);
    }
}
