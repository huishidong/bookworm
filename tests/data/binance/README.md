## websocket

[api doc](https://github.com/binance/binance-spot-api-docs/blob/master/testnet/web-socket-streams.md#how-to-manage-a-local-order-book-correctly)

The base endpoints:

`wss://data-stream.binance.vision` only market data messages
`wss://stream.testnet.binance.vision`
`wss://stream.binance.com:9443`
`wss://stream.binance.com:443`


<base>/ws/<symbol>@depth@100ms
<base>/stream?streams=<symbol0>@depth@100ms/<symbol1>@depth/
<base>/stream?streams=btcusdt@trade&timeUnit=MICROSECOND

Currently, the only property that can be set is whether combined stream payloads are enabled or not. The combined property is set to false when connecting using /ws/ ("raw streams") and true when connecting using /stream/.

### partial book depth streams

Partial Book Depth Streams
Top <levels> bids and asks, pushed every second. Valid <levels> are 5, 10, or 20.

Stream Names: <symbol>@depth<levels> OR <symbol>@depth<levels>@100ms

Update Speed: 1000ms or 100ms

Payload:

```json
{
  "lastUpdateId": 160,  // Last update ID
  "bids": [             // Bids to be updated
    [
      "0.0024",         // Price level to be updated
      "10"              // Quantity
    ]
  ],
  "asks": [             // Asks to be updated
    [
      "0.0026",         // Price level to be updated
      "100"             // Quantity
    ]
  ]
}
```

### subscription

```json
/// request
{
  "method": "SUBSCRIBE",
  "params": [
    "btcusdt@aggTrade",
    "btcusdt@depth"
  ],
  "id": 1
}

/// response
{
  "result": null,
  "id": 1
}
```

### unsubscribe

```json
{
  "method": "UNSUBSCRIBE",
  "params": [
    "btcusdt@depth"
  ],
  "id": 312
}

/// response
{
  "result": null,
  "id": 312
}
```
