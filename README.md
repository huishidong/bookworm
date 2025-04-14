# bookworm

Multi-book update aggregator and stream publisher.

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
    - [x] `test_databus` uses exchange price levels
    - [x] `test_databus` add merge both bid and ask side
    - [x] fix `bookworker`, and remove the ordering for price level in `price_level`
- [ ] put all together

## misc.
will not handle them for now.
- [ ] disconnections from exchanges
- [ ] graceful shutdown
- [ ] slow client?

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
- [ ] cli
- [ ] node.js?