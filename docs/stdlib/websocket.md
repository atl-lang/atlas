# WebSocket Functions

WebSocket client primitives built on tungstenite.

### wsConnect

```atlas
fn wsConnect(url: string) : WebSocket
```

Connects to a WebSocket server and returns a WebSocket handle.

**Parameters:**
- `url`: WebSocket URL (`ws://` or `wss://`)

**Returns:** WebSocket handle.

**Example:**
```atlas
let ws = wsConnect("wss://example.com/socket");
```

### wsSend

```atlas
fn wsSend(ws: WebSocket, message: string) : null
```

Sends a text message.

**Parameters:**
- `ws`: WebSocket handle
- `message`: Text message

**Returns:** `null`.

**Example:**
```atlas
wsSend(ws, "hello");
```

### wsSendBinary

```atlas
fn wsSendBinary(ws: WebSocket, data: number[]) : null
```

Sends binary data as an array of bytes.

**Parameters:**
- `ws`: WebSocket handle
- `data`: Array of bytes (0-255)

**Returns:** `null`.

**Example:**
```atlas
wsSendBinary(ws, [1, 2, 3]);
```

### wsReceive

```atlas
fn wsReceive(ws: WebSocket) : [string, any]
```

Receives a message and returns `[type, data]`.

**Parameters:**
- `ws`: WebSocket handle

**Returns:**
- `type` is one of `text`, `binary`, `ping`, `pong`, `close`, `frame`
- `data` is string, byte array, or null depending on type

**Example:**
```atlas
let [kind, payload] = wsReceive(ws);
```

### wsPing

```atlas
fn wsPing(ws: WebSocket) : null
```

Sends a ping frame.

**Parameters:**
- `ws`: WebSocket handle

**Returns:** `null`.

**Example:**
```atlas
wsPing(ws);
```

### wsClose

```atlas
fn wsClose(ws: WebSocket) : null
```

Closes the WebSocket connection.

**Parameters:**
- `ws`: WebSocket handle

**Returns:** `null`.

**Example:**
```atlas
wsClose(ws);
```
