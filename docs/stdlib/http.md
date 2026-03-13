# http namespace

HTTP client for making outgoing requests. Supports GET, POST, PUT, DELETE, and PATCH. Both synchronous (blocking) and asynchronous variants are available.

All functions respect the runtime security context. Requests to blocked hosts throw a permission error before the network is touched.

---

## Types

### HttpRequest

An opaque request object built with the constructor functions below. All builder functions return a new `HttpRequest` — the original is not mutated.

Default settings:
- `User-Agent: Atlas/0.1`
- Timeout: 30 seconds
- Follow redirects: true, up to 10 hops

### HttpResponse

An opaque response object returned from `httpSend` / `httpGetAsync` / etc. Inspect it with the accessor functions below.

---

## Building Requests

### httpRequest

```atlas
httpRequest(method: string, url: string): HttpRequest
```

Create a request with an explicit HTTP method. `method` must be one of `"GET"`, `"POST"`, `"PUT"`, `"DELETE"`, or `"PATCH"` (case-insensitive). `url` must start with `http://` or `https://`.

```atlas
let req = httpRequest("GET", "https://api.example.com/users");
```

### httpRequestGet

```atlas
httpRequestGet(url: string): HttpRequest
```

Convenience constructor for GET requests.

```atlas
let req = httpRequestGet("https://api.example.com/users");
```

### httpRequestPost

```atlas
httpRequestPost(url: string, body: string): HttpRequest
```

Convenience constructor for POST requests with a body.

```atlas
let req = httpRequestPost("https://api.example.com/users", `{"name":"Alice"}`);
```

### httpSetHeader

```atlas
httpSetHeader(request: HttpRequest, key: string, value: string): HttpRequest
```

Return a new request with the given header added or replaced.

```atlas
let req = httpSetHeader(req, "Content-Type", "application/json");
let req = httpSetHeader(req, "Authorization", "Bearer " + token);
```

### httpSetBody

```atlas
httpSetBody(request: HttpRequest, body: string): HttpRequest
```

Return a new request with the given body.

```atlas
let req = httpSetBody(req, `{"key":"value"}`);
```

### httpSetTimeout

```atlas
httpSetTimeout(request: HttpRequest, seconds: number): HttpRequest
```

Return a new request with the given timeout in seconds. Must be non-negative.

```atlas
let req = httpSetTimeout(req, 60);
```

---

## Sending Requests (Synchronous)

### httpSend

```atlas
httpSend(request: HttpRequest): Result<HttpResponse, string>
```

Execute the request synchronously (blocking). Returns `Ok(HttpResponse)` on success, `Err(string)` on network error.

```atlas
let result = httpSend(req);
match result {
    Ok(resp) => {
        console.log(httpStatus(resp).toString());
        console.log(httpBody(resp));
    }
    Err(e) => console.log("error: " + e),
}
```

---

## Reading Responses

### httpStatus

```atlas
httpStatus(response: HttpResponse): number
```

Return the HTTP status code (e.g. `200`, `404`).

```atlas
let code = httpStatus(resp);
```

### httpBody

```atlas
httpBody(response: HttpResponse): string
```

Return the response body as a UTF-8 string.

```atlas
let body = httpBody(resp);
```

### httpHeader

```atlas
httpHeader(response: HttpResponse, key: string): Option<string>
```

Return the value of a single response header (case-insensitive lookup), or `None` if the header is absent.

```atlas
match httpHeader(resp, "Content-Type") {
    Some(ct) => console.log(ct),
    None => console.log("no content-type"),
}
```

### httpHeaders

```atlas
httpHeaders(response: HttpResponse): HashMap<string, string>
```

Return all response headers as a map.

```atlas
let headers = httpHeaders(resp);
```

### httpUrl

```atlas
httpUrl(response: HttpResponse): string
```

Return the final URL after any redirects.

```atlas
let finalUrl = httpUrl(resp);
```

### httpIsSuccess

```atlas
httpIsSuccess(response: HttpResponse): bool
```

Return `true` if the status code is in the 200–299 range.

```atlas
if httpIsSuccess(resp) {
    processBody(httpBody(resp));
}
```

---

## Async HTTP

Async functions return `Future<HttpResponse>`. Use `await` to get the response.

### httpGetAsync

```atlas
httpGetAsync(url: string): Future<HttpResponse>
```

Perform an async GET request.

```atlas
let resp = await httpGetAsync("https://api.example.com/data");
console.log(httpBody(resp));
```

### httpPostAsync

```atlas
httpPostAsync(url: string, body: string): Future<HttpResponse>
```

Perform an async POST request with a body.

```atlas
let resp = await httpPostAsync("https://api.example.com/users", `{"name":"Bob"}`);
```

### httpPutAsync

```atlas
httpPutAsync(url: string, body: string): Future<HttpResponse>
```

Perform an async PUT request with a body.

```atlas
let resp = await httpPutAsync("https://api.example.com/users/1", `{"name":"Carol"}`);
```

### httpDeleteAsync

```atlas
httpDeleteAsync(url: string): Future<HttpResponse>
```

Perform an async DELETE request.

```atlas
let resp = await httpDeleteAsync("https://api.example.com/users/1");
```

### httpSendAsync

```atlas
httpSendAsync(request: HttpRequest): Future<HttpResponse>
```

Perform an async request from a fully-configured `HttpRequest`. Use this when you need headers, custom timeout, or query parameters.

```atlas
let req = httpRequestGet("https://api.example.com/data");
let req = httpSetHeader(req, "Authorization", "Bearer " + token);
let req = httpSetTimeout(req, 10);
let resp = await httpSendAsync(req);
```

---

## Patterns

### JSON API call

```atlas
fn fetchUser(borrow id: string): Result<string, string> {
    let req = httpRequestGet("https://api.example.com/users/" + id);
    let req = httpSetHeader(req, "Accept", "application/json");
    let result = httpSend(req);
    match result {
        Ok(resp) => {
            if httpIsSuccess(resp) {
                return Ok(httpBody(resp));
            }
            return Err("HTTP " + httpStatus(resp).toString());
        }
        Err(e) => return Err(e),
    }
}
```

### Concurrent requests

```atlas
let f1 = httpGetAsync("https://api.example.com/users");
let f2 = httpGetAsync("https://api.example.com/posts");
let both = await futureAll([f1, f2]);
let users = httpBody(both[0]);
let posts = httpBody(both[1]);
```
