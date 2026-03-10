# Network Functions

TCP, UDP, and TLS primitives with security permission checks.

### tcpConnect

```atlas
fn tcpConnect(address: string, timeout_ms?: number) : TcpStream
```

Connects to a TCP endpoint and returns a stream handle.

**Parameters:**
- `address`: Host:port or socket address
- `timeout_ms`: Optional timeout in milliseconds

**Returns:** TCP stream handle.

**Example:**
```atlas
let stream = tcpConnect("example.com:80");
```

### tcpWrite

```atlas
fn tcpWrite(stream: TcpStream, data: string) : number
```

Writes data to a TCP stream.

**Parameters:**
- `stream`: TCP stream handle
- `data`: Data to send

**Returns:** Number of bytes written.

**Example:**
```atlas
let bytes = tcpWrite(stream, "PING");
```

### tcpRead

```atlas
fn tcpRead(stream: TcpStream, max_bytes: number) : string
```

Reads up to `max_bytes` bytes from the stream and returns UTF-8 string data.

**Parameters:**
- `stream`: TCP stream handle
- `max_bytes`: Maximum bytes to read

**Returns:** String data.

**Example:**
```atlas
let data = tcpRead(stream, 1024);
```

### tcpReadBytes

```atlas
fn tcpReadBytes(stream: TcpStream, max_bytes: number) : number[]
```

Reads up to `max_bytes` bytes and returns raw bytes as numbers.

**Parameters:**
- `stream`: TCP stream handle
- `max_bytes`: Maximum bytes to read

**Returns:** Array of bytes.

**Example:**
```atlas
let bytes = tcpReadBytes(stream, 512);
```

### tcpClose

```atlas
fn tcpClose(stream: TcpStream) : null
```

Closes a TCP stream.

**Parameters:**
- `stream`: TCP stream handle

**Returns:** `null`.

**Example:**
```atlas
tcpClose(stream);
```

### tcpSetTimeout

```atlas
fn tcpSetTimeout(stream: TcpStream, read_ms: number, write_ms: number) : null
```

Sets read and write timeouts for the stream.

**Parameters:**
- `stream`: TCP stream handle
- `read_ms`: Read timeout in milliseconds
- `write_ms`: Write timeout in milliseconds

**Returns:** `null`.

**Example:**
```atlas
tcpSetTimeout(stream, 1000, 1000);
```

### tcpSetNodelay

```atlas
fn tcpSetNodelay(stream: TcpStream, nodelay: bool) : null
```

Enables or disables Nagle's algorithm.

**Parameters:**
- `stream`: TCP stream handle
- `nodelay`: `true` to disable Nagle

**Returns:** `null`.

**Example:**
```atlas
tcpSetNodelay(stream, true);
```

### tcpLocalAddr

```atlas
fn tcpLocalAddr(stream: TcpStream) : string
```

Returns the local address of the TCP stream.

**Parameters:**
- `stream`: TCP stream handle

**Returns:** Local address string.

**Example:**
```atlas
let addr = tcpLocalAddr(stream);
```

### tcpRemoteAddr

```atlas
fn tcpRemoteAddr(stream: TcpStream) : string
```

Returns the remote peer address of the TCP stream.

**Parameters:**
- `stream`: TCP stream handle

**Returns:** Remote address string.

**Example:**
```atlas
let addr = tcpRemoteAddr(stream);
```

### tcpListen

```atlas
fn tcpListen(address: string) : TcpListener
```

Binds a TCP listener to the given address.

**Parameters:**
- `address`: Bind address (host:port)

**Returns:** TCP listener handle.

**Example:**
```atlas
let listener = tcpListen("127.0.0.1:9000");
```

### tcpAccept

```atlas
fn tcpAccept(listener: TcpListener) : TcpStream
```

Accepts a new incoming connection from a TCP listener.

**Parameters:**
- `listener`: TCP listener handle

**Returns:** TCP stream handle for the accepted connection.

**Example:**
```atlas
let client = tcpAccept(listener);
```

### tcpListenerAddr

```atlas
fn tcpListenerAddr(listener: TcpListener) : string
```

Returns the local address the listener is bound to.

**Parameters:**
- `listener`: TCP listener handle

**Returns:** Bound address string.

**Example:**
```atlas
let addr = tcpListenerAddr(listener);
```

### tcpListenerClose

```atlas
fn tcpListenerClose(listener: TcpListener) : null
```

Closes a TCP listener.

**Parameters:**
- `listener`: TCP listener handle

**Returns:** `null`.

**Example:**
```atlas
tcpListenerClose(listener);
```

### udpBind

```atlas
fn udpBind(address: string) : UdpSocket
```

Binds a UDP socket to the given address.

**Parameters:**
- `address`: Bind address (host:port)

**Returns:** UDP socket handle.

**Example:**
```atlas
let socket = udpBind("0.0.0.0:9001");
```

### udpSend

```atlas
fn udpSend(socket: UdpSocket, data: string, target: string) : number
```

Sends UDP data to a target address.

**Parameters:**
- `socket`: UDP socket handle
- `data`: Data to send
- `target`: Target address

**Returns:** Number of bytes sent.

**Example:**
```atlas
udpSend(socket, "ping", "127.0.0.1:9001");
```

### udpReceive

```atlas
fn udpReceive(socket: UdpSocket, max_bytes: number) : [string, string]
```

Receives data and sender address from a UDP socket.

**Parameters:**
- `socket`: UDP socket handle
- `max_bytes`: Maximum bytes to read

**Returns:** `[data, sender]` tuple.

**Example:**
```atlas
let [data, sender] = udpReceive(socket, 2048);
```

### udpSetTimeout

```atlas
fn udpSetTimeout(socket: UdpSocket, read_ms: number) : null
```

Sets read timeout for UDP socket reads.

**Parameters:**
- `socket`: UDP socket handle
- `read_ms`: Read timeout in milliseconds

**Returns:** `null`.

**Example:**
```atlas
udpSetTimeout(socket, 500);
```

### udpClose

```atlas
fn udpClose(socket: UdpSocket) : null
```

Closes a UDP socket.

**Parameters:**
- `socket`: UDP socket handle

**Returns:** `null`.

**Example:**
```atlas
udpClose(socket);
```

### udpLocalAddr

```atlas
fn udpLocalAddr(socket: UdpSocket) : string
```

Returns the local address the UDP socket is bound to.

**Parameters:**
- `socket`: UDP socket handle

**Returns:** Local address string.

**Example:**
```atlas
let addr = udpLocalAddr(socket);
```

### tlsConnect

```atlas
fn tlsConnect(host: string, port: number) : TlsStream
```

Connects to a TLS endpoint and returns a TLS stream handle.

**Parameters:**
- `host`: Hostname
- `port`: Port number

**Returns:** TLS stream handle.

**Example:**
```atlas
let tls = tlsConnect("example.com", 443);
```

### tlsWrite

```atlas
fn tlsWrite(stream: TlsStream, data: string) : number
```

Writes data to a TLS stream.

**Parameters:**
- `stream`: TLS stream handle
- `data`: Data to send

**Returns:** Number of bytes written.

**Example:**
```atlas
tlsWrite(tls, "GET / HTTP/1.1\r\n\r\n");
```

### tlsRead

```atlas
fn tlsRead(stream: TlsStream, max_bytes: number) : string
```

Reads data from a TLS stream as a UTF-8 string.

**Parameters:**
- `stream`: TLS stream handle
- `max_bytes`: Maximum bytes to read

**Returns:** String data.

**Example:**
```atlas
let body = tlsRead(tls, 4096);
```

### tlsClose

```atlas
fn tlsClose(stream: TlsStream) : null
```

Closes a TLS stream.

**Parameters:**
- `stream`: TLS stream handle

**Returns:** `null`.

**Example:**
```atlas
tlsClose(tls);
```
