# bookworm

Multi-book update aggregator and stream publisher.
## Build and Run:

```bash
## build
cargo build

## run the app
cargo run --bin bookworm

## run client to get the summary stream data, multiple instances allowed
cargo run --example grpc_stream_client

```

## Overall Structure

As show in `main.rs`, the structure of the application consist of the following components running asynchronously:
- a gRPC server for managing client connections and publishing the summary of the aggregated book to clients
- two market data subscriptions and decoders, one for Binance, the other for Bitstamp
- a worker that stands between the market connections and the gRPC server. it receive the order book updates
  from the two market connections and perform the merging logic. the merge result is sent to gRPC output channel.

## Design Details

To simply the implementation, the app only support 2 symbols, one from each market. We are not supporting the following use cases:

1. multiple mergers running within the app.
2. combination of various book workers with different data handling logic other than the top n merge.
3. workers taking more than two market data inputs
4. one symbol used by multiple workers
5. tracing and telemetry

However, the app and its components are implemented in a way that is easily extended to support these features.

We do support multiple clients. Given the limited number of the connected clients, the data is fan-out to clients in sequence. It can be balanced if needed.

More info for each module are included in the doc strings.

## Misc.

As a starter of rust, I've tried to do a complete work and exploring the language and toolings. Various tests, documentaions, bench mark, example, third party libraries are utilized. `fmt` and `clippy` are used for formatting and linting.

### Optimization in Json deserialization

Implemented in `get_top_n_bids_asks_raw`. For large json (eg. bitstamp 100 level order book) the performance is 3x than the full deserialization.

### Merging logic.

Since the order book from the exchanges are already in the correct order, the merging logic would be as simple as pick the next best price from the two books until 10 levels or when all levels are consumed. We keep the last received books and update it as we receive new book before running the merge logic.

---

# Progress Checker

## rust fundamentals
- [x] the book
- [x] rust analyzer and vscode

## Using the protobuf and gRPC
- [x] protoc
- [x] tests apps
- [x] actual server/client

## websocket feedhandler and book aggregator
- [x] subscriber test
- [x] json deserialization optimization
- [x] book merger implementation (sort protobuf levels using binheap)
    - [x] `test_data_tunnel` uses exchange price levels
    - [x] `test_data_tunnel` add merge both bid and ask side
    - [x] fix `book_worker`, and remove the ordering for price level in `data_types`
- [x] put all together

## misc.
will not handle them for now.
- [-] force reconnection
- [-] serving multiple symbols and multiple summaries
- [-] back-pressure

## clean up
- [x] cleaned up tests and examples

### unit tests, integration tests, doc tests
- [x] included unit tests
- [x] included integration tests
- [x] no need for doc tests

### reorg modules
no need to use workspace.
- [x] done.

### metrics and performance
using `criterion` for bench marking.

- [x] benchmark json deserializers. [See Benchmark Results](benches/results.md#Conclusion)

## fancy client
- [x] cli
- [-] node.js or an webapp?