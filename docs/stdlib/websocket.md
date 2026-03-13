# websocket namespace

WebSocket client. Supports `ws://` and `wss://` (TLS) URLs. Built on the `tungstenite` library.

WebSocket handles are opaque values — pass them to the operation functions. Handles become invalid after `wsClose` is called.

---

## Connecting

### wsConnect

```atlas
wsConnect(url: string): handle
```

Connect to a WebSocket server. `url` must start with `ws://` or `wss://`. Performs the HTTP upgrade handshake synchronously. Throws on connection failure, invalid URL, or permission denial.

```atlas
let ws = wsConnect("ws://localhost:8080/chat");
let ws = wsConnect("wss://echo.websocket.org");
```

---

## Sending Messages

### wsSend

```atlas
wsSend(handle: handle, message: string): null
```

Send a text frame. The message is sent as-is (UTF-8 encoded).

```atlas
wsSend(ws, "hello server");
wsSend(ws, `{"type":"subscribe","channel":"prices"}`);
```

### wsSendBinary

```atlas
wsSendBinary(handle: handle, data: number[]): null
```

Send a binary frame. `data` must be an array of byte values (0–255).

```atlas
let bytes = [0x01, 0x02, 0x03];
wsSendBinary(ws, bytes);
```

---

## Receiving Messages

### wsReceive

```atlas
wsReceive(handle: handle): [string, string | number[] | null]
```

Block until a message arrives. Returns a two-element array `[type, data]`:

| `type`     | `data` type | Description                          |
|------------|-------------|--------------------------------------|
| `"text"`   | `string`    | UTF-8 text frame                     |
| `"binary"` | `number[]`  | Binary frame as byte array           |
| `"ping"`   | `null`      | Ping frame (reply handled by tungstenite) |
| `"pong"`   | `null`      | Pong frame                           |
| `"close"`  | `string | null` | Close frame with optional reason |
| `"frame"`  | `null`      | Raw frame (uncommon)                 |

```atlas
let msg = wsReceive(ws);
let msgType = msg[0];
let msgData = msg[1];

if msgType == "text" {
    console.log("text: " + msgData);
} else if msgType == "binary" {
    console.log("binary, " + msgData.length().toString() + " bytes");
} else if msgType == "close" {
    console.log("server closed connection");
}
```

---

## Keepalive

### wsPing

```atlas
wsPing(handle: handle): null
```

Send a WebSocket ping frame. The remote end should respond with a pong, which `wsReceive` will return as type `"pong"`.

```atlas
wsPing(ws);
```

---

## Closing

### wsClose

```atlas
wsClose(handle: handle): null
```

Send a close frame and perform the closing handshake. Reads remaining messages until the server acknowledges the close. The handle is invalid after this call.

```atlas
wsClose(ws);
```

---

## Patterns

### Message loop

```atlas
let ws = wsConnect("wss://stream.example.com/feed");
wsSend(ws, `{"action":"subscribe","symbol":"BTC"}`);

while true {
    let msg = wsReceive(ws);
    let msgType = msg[0];
    let data = msg[1];

    if msgType == "text" {
        console.log(data);
    } else if msgType == "close" {
        console.log("stream closed");
        break;
    }
}
```

### JSON message protocol

```atlas
fn sendJson(borrow ws: handle, borrow payload: string): void {
    wsSend(ws, payload);
}

fn receiveJson(borrow ws: handle): string {
    let msg = wsReceive(ws);
    if msg[0] != "text" {
        return "";
    }
    return msg[1];
}

let ws = wsConnect("wss://api.example.com/ws");
sendJson(ws, `{"event":"auth","token":"abc123"}`);
let reply = receiveJson(ws);
console.log(reply);
wsClose(ws);
```

### Ping/pong keepalive

```atlas
let ws = wsConnect("wss://api.example.com/live");
let pingCount = 0;

while true {
    // Every ~10 messages, send a ping
    let msg = wsReceive(ws);
    pingCount = pingCount + 1;
    if pingCount >= 10 {
        wsPing(ws);
        pingCount = 0;
    }
    if msg[0] == "text" {
        process(msg[1]);
    }
}
```
