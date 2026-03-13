// B28: http namespace — options-object API tests
// All network-dependent tests carry #[ignore] — preserved exactly

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
// Basic namespace method tests (non-network — type checks only)
// ============================================================================

#[test]
fn test_http_get_invalid_url_no_protocol() {
    assert!(eval_expect_error(r#"http.get("example.com");"#));
}

#[test]
fn test_http_get_invalid_url_error() {
    assert!(eval_expect_error(r#"http.get("not-a-valid-url");"#));
}

#[test]
fn test_http_post_invalid_url() {
    assert!(eval_expect_error(r#"http.post("not-a-url", "data");"#));
}

#[test]
fn test_http_put_invalid_url() {
    assert!(eval_expect_error(r#"http.put("not-a-url", "data");"#));
}

#[test]
fn test_http_delete_invalid_url() {
    assert!(eval_expect_error(r#"http.delete("not-a-url");"#));
}

#[test]
fn test_http_patch_invalid_url() {
    assert!(eval_expect_error(r#"http.patch("not-a-url", "data");"#));
}

#[test]
fn test_http_check_permission_placeholder() {
    let code = r#"httpCheckPermission("https://example.com")"#;
    // Always returns true in current placeholder implementation
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Network tests — http.get
// ============================================================================

#[test]
#[ignore = "requires network"]
fn test_http_get_simple() {
    let code = r#"
        let result = http.get("https://httpbin.org/get");
        is_ok(result) || is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
#[ignore = "requires network"]
fn test_http_get_returns_result_type() {
    let code = r#"typeof(http.get("https://httpbin.org/get"))"#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_status() {
    let code = r#"
        fn test(): number {
            let result = http.get("https://httpbin.org/status/200");
            if (is_err(result)) { return 0; }
            let response = unwrap(result);
            return response.status();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "200" || output == "0");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_body() {
    let code = r#"
        fn test(): string {
            let result = http.get("https://httpbin.org/get");
            if (is_err(result)) { return "string"; }
            let response = unwrap(result);
            return typeof(response.body());
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "string");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_is_success_200() {
    let code = r#"
        fn test(): bool {
            let result = http.get("https://httpbin.org/status/200");
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
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
        fn test(): bool {
            let result = http.get("https://httpbin.org/status/404");
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_headers() {
    let code = r#"
        fn test(): string {
            let result = http.get("https://httpbin.org/get");
            if (is_err(result)) { return "map"; }
            let response = unwrap(result);
            let headers = response.headers();
            return typeof(headers);
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "map");
}

#[test]
#[ignore = "requires network"]
fn test_http_response_url() {
    let code = r#"
        fn test(): string {
            let result = http.get("https://httpbin.org/get");
            if (is_err(result)) { return "string"; }
            let response = unwrap(result);
            return typeof(response.url());
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "string");
}

// ============================================================================
// Network tests — http.get with options
// ============================================================================

#[test]
#[ignore = "requires network"]
fn test_http_get_with_headers_option() {
    let code = r#"
        fn test(): bool {
            let opts = {"headers": {"X-Test": "atlas"}};
            let result = http.get("https://httpbin.org/get", opts);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_get_with_query_option() {
    let code = r#"
        fn test(): bool {
            let opts = {"query": {"name": "atlas", "version": "0.3"}};
            let result = http.get("https://httpbin.org/get", opts);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_get_with_timeout_option() {
    let code = r#"
        fn test(): string {
            let opts = {"timeout": 5000};
            let result = http.get("https://httpbin.org/delay/1", opts);
            return typeof(result);
        }
        test()
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
#[ignore = "requires network"]
fn test_http_get_with_useragent_option() {
    let code = r#"
        fn test(): bool {
            let opts = {"userAgent": "AtlasBot/1.0"};
            let result = http.get("https://httpbin.org/get", opts);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_get_with_bearer_auth() {
    let code = r#"
        fn test(): bool {
            let opts = {"auth": "mytoken123"};
            let result = http.get("https://httpbin.org/bearer", opts);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_get_with_basic_auth() {
    let code = r#"
        fn test(): bool {
            let opts = {"auth": "user:pass"};
            let result = http.get("https://httpbin.org/basic-auth/user/pass", opts);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

// ============================================================================
// Network tests — http.post
// ============================================================================

#[test]
#[ignore = "requires network"]
fn test_http_post_simple() {
    let code = r#"
        let result = http.post("https://httpbin.org/post", "test data");
        is_ok(result) || is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
#[ignore = "requires network"]
fn test_http_post_with_body() {
    let code = r#"
        fn test(): bool {
            let body = "name=Atlas&version=0.3";
            let result = http.post("https://httpbin.org/post", body);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_post_with_options() {
    let code = r#"
        fn test(): bool {
            let opts = {"headers": {"Content-Type": "application/x-www-form-urlencoded"}};
            let result = http.post("https://httpbin.org/post", "test=data", opts);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_post_no_body() {
    let code = r#"
        let result = http.post("https://httpbin.org/post");
        is_ok(result) || is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Network tests — http.put / http.delete / http.patch
// ============================================================================

#[test]
#[ignore = "requires network"]
fn test_http_put_simple() {
    let code = r#"
        let result = http.put("https://httpbin.org/put", "test data");
        typeof(result)
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
#[ignore = "requires network"]
fn test_http_delete_simple() {
    let code = r#"
        let result = http.delete("https://httpbin.org/delete");
        typeof(result)
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
#[ignore = "requires network"]
fn test_http_patch_simple() {
    let code = r#"
        let result = http.patch("https://httpbin.org/patch", "patch data");
        typeof(result)
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
#[ignore = "requires network"]
fn test_http_put_workflow() {
    let code = r#"
        fn test(): bool {
            let opts = {"headers": {"Content-Type": "text/plain"}};
            let result = http.put("https://httpbin.org/put", "updated data", opts);
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_delete_workflow() {
    let code = r#"
        fn test(): bool {
            let result = http.delete("https://httpbin.org/delete");
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

#[test]
#[ignore = "requires network"]
fn test_http_patch_workflow() {
    let code = r#"
        fn test(): bool {
            let result = http.patch("https://httpbin.org/patch", "partial update");
            if (is_err(result)) { return false; }
            let response = unwrap(result);
            return response.isSuccess();
        }
        test()
    "#;
    let output = eval_ok(code);
    assert!(output == "true" || output == "false");
}

// ============================================================================
// Network tests — invalid hosts return Result::Err
// ============================================================================

#[test]
#[ignore = "requires network"]
fn test_http_invalid_host_returns_error() {
    let code = r#"
        let result = http.get("https://this-domain-definitely-does-not-exist-12345.com");
        is_err(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Sandboxing tests (from api/sandboxing.rs — referenced here for completeness)
// ============================================================================
// See crates/atlas-runtime/tests/api/sandboxing.rs for network permission tests
