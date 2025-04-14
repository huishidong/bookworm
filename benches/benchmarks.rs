use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bookworm::feedhandler::*;
use bookworm::tests::TEST_DATA;

fn bench_get_top_n_binance(c: &mut Criterion) {
    c.bench_function("get_top_n_binance", |b| {
        b.iter(|| get_top_n::<bookworm::binance::OrderBook>(black_box(TEST_DATA.binance_json_byte), black_box(10)))
    });
}

fn bench_get_top_n_bids_asks_raw_binance(c: &mut Criterion) {
    c.bench_function("get_top_n_bids_asks_raw_binance", |b| {
        b.iter(|| get_top_n_bids_asks_raw(black_box(TEST_DATA.binance_json_byte), black_box(10)))
    });
}

criterion_group!(benches_binance, bench_get_top_n_binance, bench_get_top_n_bids_asks_raw_binance);

fn bench_get_top_n_bitstamp(c: &mut Criterion) {
    c.bench_function("get_top_n_bitstamp", |b| {
        b.iter(|| get_top_n::<bookworm::bitstamp::OrderBook>(black_box(TEST_DATA.bitstamp_json_byte), black_box(10)))
    });
}

fn bench_get_top_n_bids_asks_raw_bitstamp(c: &mut Criterion) {
    c.bench_function("get_top_n_bids_asks_raw_bitstamp", |b| {
        b.iter(|| get_top_n_bids_asks_raw(black_box(bookworm::bitstamp::skip_to_orderbook_data(&TEST_DATA.bitstamp_json_byte).unwrap()), black_box(10)))
    });
}

fn parse_bitstamp_instreammessage(json_str: &str) {
    let _ = serde_json::from_str::<bookworm::bitstamp::InStreamMessage>(json_str).unwrap();
}

fn bench_parse_bitstamp_instreammessage(c: &mut Criterion) {
    c.bench_function("parse_bitstamp_instreammessage", |b| {
        b.iter(|| parse_bitstamp_instreammessage(black_box(TEST_DATA.bitstamp_json_byte)))
    });
}
criterion_group!(benches_bitstamp, bench_get_top_n_bitstamp, bench_get_top_n_bids_asks_raw_bitstamp, bench_parse_bitstamp_instreammessage);

criterion_main!(benches_binance, benches_bitstamp);
