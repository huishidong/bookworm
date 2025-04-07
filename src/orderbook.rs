pub mod orderbook_proto {
    include!(concat!(env!("OUT_DIR"), "/orderbook.rs"));
}

pub use orderbook_proto::*;

pub fn print_level() {
    let level = Level {
        exchange: String::from("Binance"),
        price: 100.0,
        amount: 5.0,
    };
    println!("Level: {:?}", level);
}

pub fn print_levels() {
    let level1 = Level {
        exchange: String::from("Binance"),
        price: 100.0,
        amount: 5.0,
    };
    let level2 = Level {
        exchange: String::from("Coinbase"),
        price: 101.0,
        amount: 10.0,
    };
    let levels = vec![level1, level2];
    println!("Levels: {:?}", levels);
}

pub fn print_summary() {
    let summary = Summary {
        spread: 0.5,
        bids: vec![
            Level {
                exchange: String::from("Binance"),
                price: 100.0,
                amount: 5.0,
            },
            Level {
                exchange: String::from("Coinbase"),
                price: 101.0,
                amount: 10.0,
            },
        ],
        asks: vec![
            Level {
                exchange: String::from("Binance"),
                price: 102.0,
                amount: 5.0,
            },
            Level {
                exchange: String::from("Coinbase"),
                price: 103.0,
                amount: 10.0,
            },
        ],
    };
    println!("Summary: {:?}", summary);
}