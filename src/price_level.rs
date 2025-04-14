pub mod orderbook_proto {
    include!(concat!(env!("OUT_DIR"), "/orderbook.rs"));
}

pub use orderbook_proto::Summary;
pub use orderbook_proto::Level as ExchangePriceLevel;

pub type PxQt = [String; 2];
pub type PxQtLadder = Vec<PxQt>;
pub trait BidAsk {
    fn get_bid(&self) -> &PxQtLadder;
    fn get_ask(&self) -> &PxQtLadder;
}

impl Eq for ExchangePriceLevel {}
// impl PartialEq for ExchangePriceLevel {
//     fn eq(&self, other: &Self) -> bool {
//         self.price == other.price
//     }
// }
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
                ExchangePriceLevel {
                    exchange: String::from("Binance"),
                    price: 100.0,
                    amount: 5.0,
                },
                ExchangePriceLevel {
                    exchange: String::from("Coinbase"),
                    price: 101.0,
                    amount: 10.0,
                },
            ],
            asks: vec![
                ExchangePriceLevel {
                    exchange: String::from("Binance"),
                    price: 102.0,
                    amount: 5.0,
                },
                ExchangePriceLevel {
                    exchange: String::from("Coinbase"),
                    price: 103.0,
                    amount: 10.0,
                },
            ],
        };
        println!("Summary: {:?}", summary);
    }
}
