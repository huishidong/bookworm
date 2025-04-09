## rest api

https://www.bitstamp.net/api/v2/order_book/{market_symbol}/

## websocket

[api doc](https://www.bitstamp.net/websocket/v2/)

wss://ws.bitstamp.net

### subscription

send json

```json
{
    "event": "bts:subscribe",
    "data": {
        "channel": "[channel_name]"
    }
}
```
By changing its event property to: "bts:unsubscribe" you can delete your subscription and stop receiving events.

only list live order book related channels

### rest api order book as base line + diff order book

```js
var subscribeMsg = {
    "event": "bts:subscribe",
    "data": {
        "channel": "diff_order_book_btcusd"
    }
};
```

```js
var fetchOrderBook = function () {
    $.getJSON('https://www.bitstamp.net/api/v2/order_book/btcusd?group=1', function (data) {
        var html = '<h2>Bids</h2>';
        i = 0;

        for (i = 0; i < 500; i += 1) {
            html = html + '<div class="bid" amount="' + data.bids[i][1] + '" price="' + data.bids[i][0] + '">' + data.bids[i][1] + ' BTC @ ' + data.bids[i][0] + ' USD' + '</div>';
            bidsPlaceholder.html(html);
        }

        html = '<h2>Asks</h2>';
        for (i = 0; i < 500; i += 1) {
            html = html + '<div class="ask" amount="' + data.asks[i][1] + '" price="' + data.asks[i][0] + '">' + data.asks[i][1] + ' BTC @ ' + data.asks[i][0] + ' USD' + '</div>';
            asksPlaceholder.html(html);
        }
        orderBookTimestamp = data.microtimestamp;
    });
};
```

### live_orders_v2

channel:
```
live_orders_[currency_pair]
```

- response.event

['order_created', 'order_changed', 'order_deleted'];

- response.data
    - id	Order ID.
    - amount	Order amount.
    - amount_str	Order amount represented in string format.
    - price	Order price.
    - price_str	Order price represented in string format.
    - order_type	Order type (0 - buy; 1 - sell).
    - order_subtype	Order subtype (0 - limit; 1 - instant; 2 - market; 3 - daily; 4 - IOC; 5 - MOC; 6 - FOK; 7 - CASH SELL; 8 - GTD).
    - datetime	Order datetime.
    - microtimestamp	Order action timestamp represented in microseconds.

### heart beat

sending the following JSON message to the server

```json
{
    "event": "bts:heartbeat"
}
```

### reconnection

```json
{
    "event": "bts:request_reconnect",
    "channel": "",
    "data": ""
}
```
