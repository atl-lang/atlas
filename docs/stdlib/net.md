# net namespace

Low-level networking: TCP client and server, UDP datagrams, and TLS connections.

All operations check the security context before opening connections. Network handles are opaque values — pass them to the corresponding read/write/close functions.

---

## Handle Model

Network handles are opaque array values tagged with an internal marker string. Do not inspect or construct them directly. Always use the functions below. Handles become invalid after the corresponding close function is called.

---

## TCP Client

### tcpConnect

```atlas
tcpConnect(address: string, timeout_ms?: number): handle
```

Connect to a TCP server. `address` must be `"host:port"` format. The optional `timeout_ms` argument sets a connection timeout in milliseconds. Throws on connection failure or permission denial.

```atlas
let stream = tcpConnect("127.0.0.1:8080");
let stream = tcpConnect("api.example.com:443", 5000);
```

### tcpWrite

```atlas
tcpWrite(stream: handle, data: string): number
```

Write a UTF-8 string to the TCP stream. Returns the number of bytes written. Flushes automatically.

```atlas
let n = tcpWrite(stream, "GET / HTTP/1.0\r\n\r\n");
```

### tcpRead

```atlas
tcpRead(stream: handle, max_bytes: number): string
```

Read up to `max_bytes` bytes from the stream and return as a UTF-8 string. Capped at 1 MB per call. Throws on non-UTF-8 data.

```atlas
let response = tcpRead(stream, 4096);
```

### tcpReadBytes

```atlas
tcpReadBytes(stream: handle, max_bytes: number): number[]
```

Read up to `max_bytes` bytes from the stream and return as an array of byte values (0–255). Use when handling binary protocols.

```atlas
let raw = tcpReadBytes(stream, 1024);
```

### tcpClose

```atlas
tcpClose(stream: handle): null
```

Shut down the connection (both read and write directions).

```atlas
tcpClose(stream);
```

### tcpSetTimeout

```atlas
tcpSetTimeout(stream: handle, read_ms: number, write_ms: number): null
```

Set read and write timeouts on an existing connection in milliseconds.

```atlas
tcpSetTimeout(stream, 5000, 5000);
```

### tcpSetNodelay

```atlas
tcpSetNodelay(stream: handle, nodelay: bool): null
```

Enable or disable Nagle's algorithm. Set `true` to disable Nagle (reduce latency for small messages).

```atlas
tcpSetNodelay(stream, true);
```

### tcpLocalAddr

```atlas
tcpLocalAddr(stream: handle): string
```

Return the local socket address in `"ip:port"` format.

```atlas
let local = tcpLocalAddr(stream);
```

### tcpRemoteAddr

```atlas
tcpRemoteAddr(stream: handle): string
```

Return the remote peer address in `"ip:port"` format.

```atlas
let peer = tcpRemoteAddr(stream);
```

---

## TCP Server

### tcpListen

```atlas
tcpListen(address: string): handle
```

Bind a TCP listener to the given address. `address` must be `"host:port"` format. Use `"0.0.0.0:port"` to bind all interfaces.

```atlas
let listener = tcpListen("0.0.0.0:8080");
```

### tcpAccept

```atlas
tcpAccept(listener: handle): handle
```

Block until a new connection arrives and return a stream handle for it. Call in a loop to serve multiple clients.

```atlas
let listener = tcpListen("0.0.0.0:9000");
while true {
    let client = tcpAccept(listener);
    let data = tcpRead(client, 4096);
    tcpWrite(client, "HTTP/1.0 200 OK\r\n\r\nHello");
    tcpClose(client);
}
```

### tcpListenerAddr

```atlas
tcpListenerAddr(listener: handle): string
```

Return the bound address of the listener in `"ip:port"` format. Useful when binding to port 0 (OS-assigned port).

```atlas
let addr = tcpListenerAddr(listener);
console.log("listening on " + addr);
```

### tcpListenerClose

```atlas
tcpListenerClose(listener: handle): null
```

Stop accepting new connections and release the listener.

```atlas
tcpListenerClose(listener);
```

---

## UDP

### udpBind

```atlas
udpBind(address: string): handle
```

Bind a UDP socket to the given address. Use `"0.0.0.0:0"` to bind to an OS-assigned port.

```atlas
let socket = udpBind("0.0.0.0:5000");
```

### udpSend

```atlas
udpSend(socket: handle, data: string, target: string): number
```

Send a datagram to `target` (in `"host:port"` format). Returns the number of bytes sent.

```atlas
let n = udpSend(socket, "ping", "127.0.0.1:5001");
```

### udpReceive

```atlas
udpReceive(socket: handle, max_bytes: number): [string, string]
```

Block until a datagram arrives. Returns a two-element array `[data, sender_address]`. `max_bytes` is capped at 65535 (UDP maximum).

```atlas
let result = udpReceive(socket, 1024);
let data = result[0];
let sender = result[1];
console.log("from " + sender + ": " + data);
```

### udpSetTimeout

```atlas
udpSetTimeout(socket: handle, read_ms: number): null
```

Set a read timeout on the UDP socket in milliseconds.

```atlas
udpSetTimeout(socket, 2000);
```

### udpLocalAddr

```atlas
udpLocalAddr(socket: handle): string
```

Return the local bound address in `"ip:port"` format.

```atlas
let addr = udpLocalAddr(socket);
```

### udpClose

```atlas
udpClose(socket: handle): null
```

Close the UDP socket.

```atlas
udpClose(socket);
```

---

## TLS Client

TLS connections are layered on top of TCP and use the system's trusted root certificates (via `webpki-roots`). Client certificates are not currently supported.

### tlsConnect

```atlas
tlsConnect(host: string, port: number): handle
```

Open a TLS connection to `host:port`. Performs certificate validation against trusted roots. Throws on connection failure or invalid certificate.

```atlas
let stream = tlsConnect("api.example.com", 443);
```

### tlsWrite

```atlas
tlsWrite(stream: handle, data: string): number
```

Write a UTF-8 string through the TLS stream. Returns bytes written.

```atlas
tlsWrite(stream, "GET / HTTP/1.1\r\nHost: api.example.com\r\n\r\n");
```

### tlsRead

```atlas
tlsRead(stream: handle, max_bytes: number): string
```

Read up to `max_bytes` bytes from the TLS stream. Returns a UTF-8 string, capped at 1 MB per call.

```atlas
let resp = tlsRead(stream, 8192);
```

### tlsClose

```atlas
tlsClose(stream: handle): null
```

Send TLS close-notify and close the connection.

```atlas
tlsClose(stream);
```

---

## Patterns

### Echo server

```atlas
let listener = tcpListen("127.0.0.1:9999");
console.log("echo server on " + tcpListenerAddr(listener));
while true {
    let client = tcpAccept(listener);
    let data = tcpRead(client, 4096);
    tcpWrite(client, data);
    tcpClose(client);
}
```

### Simple HTTPS GET

```atlas
let stream = tlsConnect("httpbin.org", 443);
tlsWrite(stream, "GET /get HTTP/1.0\r\nHost: httpbin.org\r\n\r\n");
let response = tlsRead(stream, 16384);
tlsClose(stream);
console.log(response);
```
