# Benchmark Results

json deserialization benchmarking.

## binance data

we can subscribe to 10 level only.

### naive

```bash
Benchmarking get_top_n_binance
Benchmarking get_top_n_binance: Warming up for 3.0000 s
Benchmarking get_top_n_binance: Collecting 100 samples in estimated 5.0061 s (2.4M iterations)
Benchmarking get_top_n_binance: Analyzing
get_top_n_binance       time:   [2.0543 µs 2.0918 µs 2.1312 µs]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
```

### customized visitor

```bash
Benchmarking get_top_n_bids_asks_raw_binance
Benchmarking get_top_n_bids_asks_raw_binance: Warming up for 3.0000 s
Benchmarking get_top_n_bids_asks_raw_binance: Collecting 100 samples in estimated 5.0064 s (3.1M iterations)
Benchmarking get_top_n_bids_asks_raw_binance: Analyzing
get_top_n_bids_asks_raw_binance
                        time:   [1.6172 µs 1.6331 µs 1.6528 µs]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
```

## bitstamp data

where 100 level order book updates are being processed.

### naive

```bash
Benchmarking get_top_n_bitstamp
Benchmarking get_top_n_bitstamp: Warming up for 3.0000 s
Benchmarking get_top_n_bitstamp: Collecting 100 samples in estimated 5.0395 s (293k iterations)
Benchmarking get_top_n_bitstamp: Analyzing
get_top_n_bitstamp      time:   [16.530 µs 16.669 µs 16.825 µs]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low mild
  6 (6.00%) high mild
  4 (4.00%) high severe
```

### customized visitor

```bash
Benchmarking get_top_n_bids_asks_raw_bitstamp
Benchmarking get_top_n_bids_asks_raw_bitstamp: Warming up for 3.0000 s
Benchmarking get_top_n_bids_asks_raw_bitstamp: Collecting 100 samples in estimated 5.0262 s (929k iterations)
Benchmarking get_top_n_bids_asks_raw_bitstamp: Analyzing
get_top_n_bids_asks_raw_bitstamp
                        time:   [5.4916 µs 5.6444 µs 5.8548 µs]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
```
## Conclusion

Our customized deserializer gives the best performance for orderbook updates from both exchanges.