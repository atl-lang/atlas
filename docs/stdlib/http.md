# HTTP Functions

HTTP client requests and response handling.

## Request Building

### httpRequest

```atlas
fn httpRequest(method: string, url: string) -> HttpRequest
```

Creates a new HTTP request builder.

**Parameters:**
- `method` - HTTP method (GET, POST, PUT, PATCH, DELETE)
- `url` - Request URL

**Returns:** `HttpRequest` - Request builder for chaining

## HTTP Methods

### httpGet

```atlas
fn httpGet(url: string) -> Result<HttpResponse, string>
```

Sends GET request.

**Parameters:**
- `url` - Request URL

**Returns:**
- `Ok(HttpResponse)` on success
- `Err(string)` on failure

**Alias:** `httpRequestGet`

### httpPost

```atlas
fn httpPost(url: string, body: string) -> Result<HttpResponse, string>
```

Sends POST request.

**Parameters:**
- `url` - Request URL
- `body` - Request body

**Returns:**
- `Ok(HttpResponse)` on success
- `Err(string)` on failure

**Builder:** `httpRequestPost`

### httpPut

```atlas
fn httpPut(url: string, body: string) -> Result<HttpResponse, string>
```

Sends PUT request.

**Parameters:**
- `url` - Request URL
- `body` - Request body

**Returns:**
- `Ok(HttpResponse)` on success
- `Err(string)` on failure

**Builder:** `httpRequestPut`

### httpPatch

```atlas
fn httpPatch(url: string, body: string) -> Result<HttpResponse, string>
```

Sends PATCH request.

**Parameters:**
- `url` - Request URL
- `body` - Request body

**Returns:**
- `Ok(HttpResponse)` on success
- `Err(string)` on failure

**Builder:** `httpRequestPatch`

### httpDelete

```atlas
fn httpDelete(url: string) -> Result<HttpResponse, string>
```

Sends DELETE request.

**Parameters:**
- `url` - Request URL

**Returns:**
- `Ok(HttpResponse)` on success
- `Err(string)` on failure

**Builder:** `httpRequestDelete`

## Request Configuration

### httpSetBody

```atlas
fn httpSetBody(request: HttpRequest, body: string) -> HttpRequest
```

Sets request body.

**Parameters:**
- `request` - HTTP request
- `body` - Body content

**Returns:** `HttpRequest` - Request with body set

### httpSetHeader

```atlas
fn httpSetHeader(request: HttpRequest, key: string, value: string) -> HttpRequest
```

Sets HTTP header.

**Parameters:**
- `request` - HTTP request
- `key` - Header name
- `value` - Header value

**Returns:** `HttpRequest` - Request with header added

### httpSetQuery

```atlas
fn httpSetQuery(request: HttpRequest, key: string, value: string) -> HttpRequest
```

Sets query parameter.

**Parameters:**
- `request` - HTTP request
- `key` - Parameter name
- `value` - Parameter value

**Returns:** `HttpRequest` - Request with query parameter added

### httpSetAuth

```atlas
fn httpSetAuth(request: HttpRequest, username: string, password: string) -> HttpRequest
```

Sets Basic authentication.

**Parameters:**
- `request` - HTTP request
- `username` - Username
- `password` - Password

**Returns:** `HttpRequest` - Request with auth set

### httpSetUserAgent

```atlas
fn httpSetUserAgent(request: HttpRequest, user_agent: string) -> HttpRequest
```

Sets User-Agent header.

**Parameters:**
- `request` - HTTP request
- `user_agent` - User-Agent value

**Returns:** `HttpRequest` - Request with User-Agent set

### httpSetTimeout

```atlas
fn httpSetTimeout(request: HttpRequest, seconds: number) -> HttpRequest
```

Sets request timeout.

**Parameters:**
- `request` - HTTP request
- `seconds` - Timeout in seconds

**Returns:** `HttpRequest` - Request with timeout set

### httpSetFollowRedirects

```atlas
fn httpSetFollowRedirects(request: HttpRequest, follow: bool) -> HttpRequest
```

Enables or disables redirect following.

**Parameters:**
- `request` - HTTP request
- `follow` - True to follow redirects

**Returns:** `HttpRequest` - Request with setting updated

### httpSetMaxRedirects

```atlas
fn httpSetMaxRedirects(request: HttpRequest, max: number) -> HttpRequest
```

Sets maximum redirects to follow.

**Parameters:**
- `request` - HTTP request
- `max` - Maximum redirect count (integer)

**Returns:** `HttpRequest` - Request with max redirects set

## Sending Requests

### httpSend

```atlas
fn httpSend(request: HttpRequest) -> Result<HttpResponse, string>
```

Sends HTTP request.

**Parameters:**
- `request` - Configured HTTP request

**Returns:**
- `Ok(HttpResponse)` on success
- `Err(string)` on network/timeout error

### httpSendAsync

```atlas
fn httpSendAsync(request: HttpRequest) -> Future<Result<HttpResponse, string>>
```

Sends HTTP request asynchronously.

**Parameters:**
- `request` - Configured HTTP request

**Returns:** `Future` - Resolves to response or error

## Convenience Methods

### httpGetJson

```atlas
fn httpGetJson(url: string) -> Result<json, string>
```

GET request expecting JSON response. Parses automatically.

**Parameters:**
- `url` - Request URL

**Returns:**
- `Ok(json)` - Parsed JSON
- `Err(string)` on failure or parse error

### httpPostJson

```atlas
fn httpPostJson(url: string, data: json) -> Result<HttpResponse, string>
```

POST request with JSON body.

**Parameters:**
- `url` - Request URL
- `data` - JSON value to send

**Returns:**
- `Ok(HttpResponse)` on success
- `Err(string)` on failure

**Note:** Use `httpParseJson(response)` to parse the response body.

### httpPostAsync

```atlas
fn httpPostAsync(url: string, body: string) -> Future<Result<HttpResponse, string>>
```

POST request asynchronously.

**Parameters:**
- `url` - Request URL
- `body` - Request body

**Returns:** `Future`

### httpGetAsync

```atlas
fn httpGetAsync(url: string) -> Future<Result<HttpResponse, string>>
```

GET request asynchronously.

**Parameters:**
- `url` - Request URL

**Returns:** `Future`

### httpDeleteAsync

```atlas
fn httpDeleteAsync(url: string) -> Future<Result<HttpResponse, string>>
```

DELETE request asynchronously.

**Parameters:**
- `url` - Request URL

**Returns:** `Future`

### httpPutAsync

```atlas
fn httpPutAsync(url: string, body: string) -> Future<Result<HttpResponse, string>>
```

PUT request asynchronously.

**Parameters:**
- `url` - Request URL
- `body` - Request body

**Returns:** `Future`

## Response Handling

### httpStatus

```atlas
fn httpStatus(response: HttpResponse) -> number
```

Gets HTTP status code.

**Parameters:**
- `response` - HTTP response

**Returns:** `number` - Status code (e.g., 200, 404)

### httpStatusText

```atlas
fn httpStatusText(response: HttpResponse) -> string
```

Gets HTTP status text.

**Parameters:**
- `response` - HTTP response

**Returns:** `string` - Status text (e.g., "OK", "Not Found")

### httpBody

```atlas
fn httpBody(response: HttpResponse) -> string
```

Gets response body as string.

**Parameters:**
- `response` - HTTP response

**Returns:** `string` - Response body

### httpContentType

```atlas
fn httpContentType(response: HttpResponse) -> Option<string>
```

Gets Content-Type header value.

**Parameters:**
- `response` - HTTP response

**Returns:** `Option<string>` - Content-Type or None

### httpContentLength

```atlas
fn httpContentLength(response: HttpResponse) -> Option<number>
```

Gets Content-Length header value.

**Parameters:**
- `response` - HTTP response

**Returns:** `Option<number>` - Content length or None

### httpHeader

```atlas
fn httpHeader(response: HttpResponse, key: string) -> Option<string>
```

Gets header value by name.

**Parameters:**
- `response` - HTTP response
- `key` - Header name

**Returns:** `Option<string>` - Header value or None

### httpHeaders

```atlas
fn httpHeaders(response: HttpResponse) -> HashMap<string, string>
```

Gets all headers as a HashMap.

**Parameters:**
- `response` - HTTP response

**Returns:** `object` - HashMap of headers

## Status Checking

### httpIsSuccess

```atlas
fn httpIsSuccess(response: HttpResponse) -> bool
```

Checks if status code is 2xx (success).

**Parameters:**
- `response` - HTTP response

**Returns:** `bool`

### httpIsRedirect

```atlas
fn httpIsRedirect(response: HttpResponse) -> bool
```

Checks if status code is 3xx (redirect).

**Parameters:**
- `response` - HTTP response

**Returns:** `bool`

### httpIsClientError

```atlas
fn httpIsClientError(response: HttpResponse) -> bool
```

Checks if status code is 4xx (client error).

**Parameters:**
- `response` - HTTP response

**Returns:** `bool`

### httpIsServerError

```atlas
fn httpIsServerError(response: HttpResponse) -> bool
```

Checks if status code is 5xx (server error).

**Parameters:**
- `response` - HTTP response

**Returns:** `bool`

## Security

### httpCheckPermission

```atlas
fn httpCheckPermission(url: string) -> Result<bool, string>
```

Checks if URL is allowed by security policy.

**Parameters:**
- `url` - URL to check

**Returns:**
- `Ok(bool)` - True if allowed
- `Err(string)` on error

### httpParseJson

```atlas
fn httpParseJson(response: HttpResponse) -> Result<any, string>
```

Parses response body as JSON.

**Parameters:**
- `response` - HTTP response

**Returns:**
- `Ok(any)` - Parsed JSON
- `Err(string)` if invalid JSON

## Example Usage

```atlas
// Simple GET
let response = httpGet("https://api.example.com/users")?;
print(httpBody(response));

// POST with JSON
let req = httpRequest("POST", "https://api.example.com/users")
  |> httpSetBody("{\"name\": \"John\"}")
  |> httpSetHeader("Content-Type", "application/json");
let response = httpSend(req)?;
print(httpStatus(response)); // 201

// GET with auth
let req = httpGet("https://api.example.com/secret")
  |> httpSetAuth("user", "pass");
let response = httpSend(req)?;
```
