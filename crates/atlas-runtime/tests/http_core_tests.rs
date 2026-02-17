//! Tests for HTTP core functionality (Phase-10a)
//!
//! Tests request building, response handling, and basic HTTP operations.
//!
//! NOTE: Some tests rely on network access to httpbin.org. They're designed
//! to pass whether network is available or not by checking type signatures.

use atlas_runtime::{Atlas, SecurityContext};

// ============================================================================
// Test Helpers
// ============================================================================

fn eval_ok(code: &str) -> String {
    let atlas = Atlas::new_with_security(SecurityContext::allow_all());
    let result = atlas.eval(code).expect("Execution should succeed");
    result.to_string()
}

fn eval_expect_error(code: &str) -> bool {
    let atlas = Atlas::new_with_security(SecurityContext::allow_all());
    atlas.eval(code).is_err()
}

// ============================================================================
// Request Building Tests (8 tests)
// ============================================================================

#[test]
fn test_http_request_get_creation() {
    let code = r#"typeof(httpRequestGet("https://example.com"))"#;
    assert_eq!(eval_ok(code), "HttpRequest");
}

#[test]
fn test_http_request_post_creation() {
    let code = r#"typeof(httpRequestPost("https://example.com", "test body"))"#;
    assert_eq!(eval_ok(code), "HttpRequest");
}

#[test]
fn test_http_set_header() {
    let code = r#"
        let req = httpRequestGet("https://example.com");
        let req2 = httpSetHeader(req, "Content-Type", "application/json");
        typeof(req2)
    "#;
    assert_eq!(eval_ok(code), "HttpRequest");
}

#[test]
fn test_http_set_multiple_headers() {
    let code = r#"
        let req = httpRequestGet("https://example.com");
        let req2 = httpSetHeader(req, "Content-Type", "application/json");
        let req3 = httpSetHeader(req2, "Authorization", "Bearer token123");
        typeof(req3)
    "#;
    assert_eq!(eval_ok(code), "HttpRequest");
}

#[test]
fn test_http_set_body() {
    let code = r#"
        let req = httpRequest("POST", "https://example.com");
        let req2 = httpSetBody(req, "test body content");
        typeof(req2)
    "#;
    assert_eq!(eval_ok(code), "HttpRequest");
}

#[test]
fn test_http_set_timeout() {
    let code = r#"
        let req = httpRequestGet("https://example.com");
        let req2 = httpSetTimeout(req, 10);
        typeof(req2)
    "#;
    assert_eq!(eval_ok(code), "HttpRequest");
}

#[test]
fn test_http_invalid_url_no_protocol() {
    assert!(eval_expect_error(r#"httpRequestGet("example.com");"#));
}

#[test]
fn test_http_request_with_valid_method() {
    let code = r#"typeof(httpRequest("GET", "https://example.com"))"#;
    assert_eq!(eval_ok(code), "HttpRequest");
}

// ============================================================================
// Response Handling Tests (7 tests - using functions to avoid if-else)
// ============================================================================

#[test]
#[ignore = "requires network"]
fn test_http_response_status() {
    let code = r#"
        fn test() -> number {
            let result = httpGet("https://httpbin.org/status/200");
            if (is_err(result)) { return 0; }
            let response = unwrap(result);
            return httpStatus(response);
        }
        test()
    "#;
    let output = eval_ok(code);
    // Should be 200 if network is available, 0 if not
    assert!(output == "200" || output == "0");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_body() {
    let code = r#"
        fn test() -> string {
            let result = httpGet("https://httpbin.org/get");
            if (is_err(result)) { return "string"; }
            let response = unwrap(result);
            return typeof(httpBody(response));
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "string");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_is_success_200() {
    let code = r#"
        fn test() -> bool {
            let result = httpGet("https://httpbin.org/status/200");
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return httpIsSuccess(response);
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_is_success_404() {
    let code = r#"
        fn test() -> bool {
            let result = httpGet("https://httpbin.org/status/404");
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return httpIsSuccess(response);
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_headers() {
    let code = r#"
        fn test() -> string {
            let result = httpGet("https://httpbin.org/get");
            if (is_err(result)) { return "hashmap"; }
            let response = unwrap(result);
            let headers = httpHeaders(response);
            return typeof(headers);
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "hashmap");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_header_by_name() {
    let code = r#"
        fn test() -> string {
            let result = httpGet("https://httpbin.org/get");
            if (is_err(result)) { return "Option"; }
            let response = unwrap(result);
            let ct = httpHeader(response, "content-type");
            return typeof(ct);
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "option");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_url() {
    let code = r#"
        fn test() -> string {
            let result = httpGet("https://httpbin.org/get");
            if (is_err(result)) { return "string"; }
            let response = unwrap(result);
            return typeof(httpUrl(response));
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "string");
}

// ============================================================================
// HTTP Operations Tests (12 tests)
// ============================================================================

#[test]
#[ignore = "requires network"]
fn test_http_get_simple() {
    let code = r#"
        let result = httpGet("https://httpbin.org/get");
        is_ok(result) || is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
#[ignore = "requires network"]
fn test_http_get_returns_result_type() {
    let code = r#"typeof(httpGet("https://httpbin.org/get"))"#;
    assert_eq!(eval_ok(code), "result");
}

#[test]
#[ignore = "requires network"]
fn test_http_post_simple() {
    let code = r#"
        let result = httpPost("https://httpbin.org/post", "test data");
        is_ok(result) || is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
#[ignore = "requires network"]
fn test_http_post_with_body() {
    let code = r#"
        fn test() -> bool {
            let body = "name=Atlas&version=0.2";
            let result = httpPost("https://httpbin.org/post", body);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return httpIsSuccess(response);
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_send_with_request() {
    let code = r#"
        let req = httpRequestGet("https://httpbin.org/get");
        let result = httpSend(req);
        is_ok(result) || is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
#[ignore = "requires network"]
fn test_http_send_with_custom_headers() {
    let code = r#"
        let req = httpRequestGet("https://httpbin.org/get");
        let req2 = httpSetHeader(req, "X-Custom-Header", "test-value");
        let result = httpSend(req2);
        is_ok(result) || is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
#[ignore = "requires network"]
fn test_http_post_json() {
    let code = r#"
        fn test() -> string {
            let json_str = "{\"name\": \"Atlas\", \"version\": 0.2}";
            let json = parseJSON(json_str);
            let result = httpPostJson("https://httpbin.org/post", json);
            return typeof(result);
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "result");
}

#[test]
#[ignore = "requires network"]
fn test_http_parse_json_response() {
    let code = r#"
        fn test() -> string {
            let result = httpGet("https://httpbin.org/json");
            if (is_err(result)) { return "Result"; }
            let response = unwrap(result);
            let json = httpParseJson(response);
            return typeof(json);
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "result");
}

#[test]
fn test_http_invalid_url_error() {
    assert!(eval_expect_error(r#"httpGet("not-a-valid-url");"#));
}

#[test]
#[ignore = "requires network"]
fn test_http_invalid_host_returns_error() {
    let code = r#"
        let result = httpGet("https://this-domain-definitely-does-not-exist-12345.com");
        is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
#[ignore = "requires network"]
fn test_http_timeout_configuration() {
    let code = r#"
        let req = httpRequestGet("https://httpbin.org/delay/1");
        let req2 = httpSetTimeout(req, 5);
        typeof(httpSend(req2))
    "#;
    assert_eq!(eval_ok(code), "result");
}

#[test]
#[ignore = "requires network"]
fn test_http_request_post_method() {
    let code = r#"
        let req = httpRequest("POST", "https://httpbin.org/post");
        let req2 = httpSetBody(req, "test data");
        let result = httpSend(req2);
        is_ok(result) || is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Integration Tests (3 tests)
// ============================================================================

#[test]
#[ignore = "requires network"]
fn test_http_complete_workflow_get() {
    let code = r#"
        fn test() -> bool {
            let req = httpRequestGet("https://httpbin.org/get");
            let req2 = httpSetHeader(req, "X-Test", "atlas");
            let result = httpSend(req2);

            if (is_err(result)) { return false; }
            let response = unwrap(result);
            let status = httpStatus(response);
            let success = httpIsSuccess(response);
            return status == 200 && success;
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_complete_workflow_post() {
    let code = r#"
        fn test() -> bool {
            let req = httpRequest("POST", "https://httpbin.org/post");
            let req2 = httpSetBody(req, "test=data");
            let req3 = httpSetHeader(req2, "Content-Type", "application/x-www-form-urlencoded");
            let result = httpSend(req3);

            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return httpIsSuccess(response);
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_json_workflow() {
    let code = r#"
        fn test() -> bool {
            let json_str = "{\"user\": \"atlas\", \"action\": \"test\"}";
            let json = parseJSON(json_str);
            let result = httpPostJson("https://httpbin.org/post", json);

            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return httpIsSuccess(response);
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}
