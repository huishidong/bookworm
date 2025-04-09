fn main() {
 /*
    1) gRPC in RUST
    2) Protobuf
    3) gRPC Server
    4) gRPC Client
 */
}

// //! A full working Bitstamp order book merger app with metrics

// use futures_util::{SinkExt, StreamExt};
// use serde::Deserialize;
// use std::collections::BinaryHeap;
// use std::cmp::Reverse;
// use tokio::{sync::mpsc, time::{self, Duration}};
// use tokio_tungstenite::{connect_async, tungstenite::Message};
// use axum::{Router, routing::get};
// use std::net::SocketAddr;
// use prometheus::{IntCounter, Gauge, Registry, Encoder, TextEncoder};

// #[derive(Debug, Clone)]
// pub struct PriceLevel {
//     pub price: f64,
//     pub amount: f64,
//     pub side: Side,
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum Side {
//     Bid,
//     Ask,
// }

// impl PartialOrd for PriceLevel {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for PriceLevel {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         match self.side {
//             Side::Bid => other.price.partial_cmp(&self.price).unwrap(), // descending
//             Side::Ask => self.price.partial_cmp(&other.price).unwrap(), // ascending
//         }
//     }
// }

// #[derive(Debug, Deserialize)]
// struct OrderBookMessage {
//     data: OrderBookData,
//     event: String,
//     channel: String,
// }

// #[derive(Debug, Deserialize)]
// struct OrderBookData {
//     bids: Vec<[String; 2]>,
//     asks: Vec<[String; 2]>,
// }

// #[derive(Clone)]
// pub struct Metrics {
//     pub registry: Registry,
//     pub levels_received: IntCounter,
//     pub last_bid_price: Gauge,
//     pub last_ask_price: Gauge,
// }

// impl Metrics {
//     pub fn new() -> Self {
//         let registry = Registry::new();

//         let levels_received = IntCounter::new("price_levels_received_total", "Total price levels received").unwrap();
//         let last_bid_price = Gauge::new("last_bid_price", "Latest top-of-book bid").unwrap();
//         let last_ask_price = Gauge::new("last_ask_price", "Latest top-of-book ask").unwrap();

//         registry.register(Box::new(levels_received.clone())).unwrap();
//         registry.register(Box::new(last_bid_price.clone())).unwrap();
//         registry.register(Box::new(last_ask_price.clone())).unwrap();

//         Self {
//             registry,
//             levels_received,
//             last_bid_price,
//             last_ask_price,
//         }
//     }

//     pub fn gather(&self) -> String {
//         let encoder = TextEncoder::new();
//         let metric_families = self.registry.gather();
//         let mut buf = Vec::new();
//         encoder.encode(&metric_families, &mut buf).unwrap();
//         String::from_utf8(buf).unwrap()
//     }
// }

// #[tokio::main]
// async fn main() {
//     let metrics = Metrics::new();
//     let (tx, rx) = mpsc::unbounded_channel();

//     let metrics_clone = metrics.clone();
//     tokio::spawn(async move {
//         loop {
//             if let Err(e) = start_bitstamp_producer(tx.clone()).await {
//                 eprintln!("[Producer] Error: {e}, retrying in 3s...");
//                 time::sleep(Duration::from_secs(3)).await;
//             }
//         }
//     });

//     let metrics_clone2 = metrics.clone();
//     tokio::spawn(async move {
//         merger(rx, metrics_clone2).await;
//     });

//     let app = Router::new().route("/metrics", get(move || async move {
//         metrics_clone.gather()
//     }));

//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//     println!("\u{1f4ca} Serving metrics on http://{addr}/metrics");
//     axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
// }

// pub async fn start_bitstamp_producer(tx: mpsc::UnboundedSender<PriceLevel>) -> Result<(), Box<dyn std::error::Error>> {
//     let url = "wss://ws.bitstamp.net";
//     let (ws_stream, _) = connect_async(url).await?;
//     println!("\u{1f7e2} Connected to Bitstamp WS");

//     let (mut write, mut read) = ws_stream.split();

//     let sub_msg = serde_json::json!({
//         "event": "bts:subscribe",
//         "data": {
//             "channel": "order_book_btcusd"
//         }
//     });
//     write.send(Message::Text(sub_msg.to_string())).await?;

//     while let Some(msg) = read.next().await {
//         match msg {
//             Ok(Message::Text(text)) => {
//                 if let Ok(ob_msg) = serde_json::from_str::<OrderBookMessage>(&text) {
//                     for [price, amount] in ob_msg.data.bids.iter().take(10) {
//                         if let (Ok(p), Ok(a)) = (price.parse(), amount.parse()) {
//                             let _ = tx.send(PriceLevel { price: p, amount: a, side: Side::Bid });
//                         }
//                     }
//                     for [price, amount] in ob_msg.data.asks.iter().take(10) {
//                         if let (Ok(p), Ok(a)) = (price.parse(), amount.parse()) {
//                             let _ = tx.send(PriceLevel { price: p, amount: a, side: Side::Ask });
//                         }
//                     }
//                 }
//             }
//             Ok(Message::Ping(_)) => {}
//             Err(e) => {
//                 return Err(e.into());
//             }
//             _ => {}
//         }
//     }

//     Ok(())
// }

// async fn merger(mut rx: mpsc::UnboundedReceiver<PriceLevel>, metrics: Metrics) {
//     let mut bids = BinaryHeap::new();
//     let mut asks = BinaryHeap::new();

//     loop {
//         if let Some(level) = rx.recv().await {
//             metrics.levels_received.inc();

//             match level.side {
//                 Side::Bid => {
//                     metrics.last_bid_price.set(level.price);
//                     bids.push(Reverse(level));
//                     if bids.len() > 10 { bids.pop(); }
//                 }
//                 Side::Ask => {
//                     metrics.last_ask_price.set(level.price);
//                     asks.push(Reverse(level));
//                     if asks.len() > 10 { asks.pop(); }
//                 }
//             }
//         }
//     }
// }
