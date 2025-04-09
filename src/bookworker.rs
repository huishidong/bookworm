use crate::price_level::ExchangePriceLevel;
use crate::databus::PriceLevelReceiver;
use std::cmp::Reverse;
use std::collections::BinaryHeap;


pub async fn merger(mut rx_a: PriceLevelReceiver, mut rx_b: PriceLevelReceiver) 
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut merged_bids: BinaryHeap<Reverse<ExchangePriceLevel>> = BinaryHeap::new();
    let mut merged_asks: BinaryHeap<ExchangePriceLevel> = BinaryHeap::new();

    loop {
        tokio::select! {
            Some(level_a) = rx_a.recv() => {
                for level in level_a.0 {
                    merged_bids.push(Reverse(ExchangePriceLevel{exchange: "Binance".to_string(), price: level[0].parse()?, amount: level[1].parse()?}));
                }
                for level in level_a.1 {
                    merged_asks.push(ExchangePriceLevel{exchange: "Binance".to_string(), price: level[0].parse()?, amount: level[1].parse()?});
                }
            }
            Some(level_b) = rx_b.recv() => {
                for level in level_b.0 {
                    merged_bids.push(Reverse(ExchangePriceLevel{exchange: "Bitstamp".to_string(), price: level[0].parse()?, amount: level[1].parse()?}));
                }
                for level in level_b.1 {
                    merged_asks.push(ExchangePriceLevel{exchange: "Bitstamp".to_string(), price: level[0].parse()?, amount: level[1].parse()?});
                }
            }
            else => break,
        }

        while merged_bids.len() > 10 {
            merged_bids.pop(); // Keep top N
        }

        while merged_asks.len() > 10 {
            merged_bids.pop(); // Keep top N
        }

        for level in &merged_bids {
            println!("Bid: {} - ${:.6} ({:.6})", level.0.exchange, level.0.price, level.0.amount);
        }
        for level in &merged_asks {
            println!("Ask: {} - ${:.6} ({:.6})", level.exchange, level.price, level.amount);
        }
    }
    Ok(())
}
