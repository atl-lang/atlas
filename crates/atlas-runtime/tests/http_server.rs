// http.serve() HTTP server primitive tests
// Server integration tests are gated by ATLAS_TEST_SERVER=1 env var.
// Enable with:
//   ATLAS_TEST_SERVER=1 cargo nextest run --test http_server --run-ignored
//
// Unit tests (arity/type checks) run always — they do NOT start a real server.

use atlas_runtime::{Atlas, SecurityContext};

/// Runtime guard — call at start of every server integration test body.
/// If ATLAS_TEST_SERVER is not set, returns early (test passes as no-op).
macro_rules! require_server {
    () => {
        if std::env::var("ATLAS_TEST_SERVER").unwrap_or_default() != "1" {
            return;
        }
    };
}

// ============================================================================
// Test Helpers
// ============================================================================

fn eval_expect_error(code: &str) -> bool {
    let atlas = Atlas::new_with_security(SecurityContext::allow_all());
    atlas.eval(code).is_err()
}

#[cfg(feature = "http")]
fn make_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("Failed to build reqwest client")
}

// ============================================================================
// Unit tests — arity and type validation (no real server)
// ============================================================================

#[test]
fn test_http_serve_no_args() {
    assert!(eval_expect_error(r#"http.serve();"#));
}

#[test]
fn test_http_serve_one_arg() {
    assert!(eval_expect_error(r#"http.serve(8080);"#));
}

#[test]
fn test_http_serve_three_args() {
    assert!(eval_expect_error(
        r#"http.serve(8080, fn(req) { "ok" }, "extra");"#
    ));
}

#[test]
fn test_http_serve_invalid_port_type() {
    assert!(eval_expect_error(
        r#"http.serve("8080", fn(req) { "ok" });"#
    ));
}

#[test]
fn test_http_serve_invalid_handler_type() {
    assert!(eval_expect_error(r#"http.serve(8080, "not-a-fn");"#));
}

#[test]
fn test_http_serve_null_handler() {
    assert!(eval_expect_error(r#"http.serve(8080, null);"#));
}

// ============================================================================
// Integration tests — real HTTP server (gated by ATLAS_TEST_SERVER=1)
// ============================================================================

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_basic_200() {
    require_server!();

    let port = 18001u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 200, "body": "hello"}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "hello");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_plain_string() {
    require_server!();

    let port = 18002u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return "hi"; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "hi");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_custom_status_404() {
    require_server!();

    let port = 18003u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 404, "body": "not found"}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 404);
    assert_eq!(resp.text().unwrap(), "not found");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_read_method_get() {
    require_server!();

    let port = 18004u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 200, "body": req.method}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "GET");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_read_method_post() {
    require_server!();

    let port = 18005u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 200, "body": req.method}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .post(format!("http://127.0.0.1:{}/", port))
        .body("")
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "POST");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_read_path() {
    require_server!();

    let port = 18006u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 200, "body": req.path}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/foo/bar", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "/foo/bar");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_read_body() {
    require_server!();

    let port = 18007u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 200, "body": req.body}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .post(format!("http://127.0.0.1:{}/", port))
        .body("hello world")
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "hello world");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_json_response() {
    require_server!();

    let port = 18008u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{
                return {{
                    "status": 200,
                    "body": "{{\\"name\\":\\"atlas\\"}}",
                    "headers": {{"Content-Type": "application/json"}}
                }};
            }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("application/json"),
        "expected application/json, got: {}",
        ct
    );
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_custom_headers() {
    require_server!();

    let port = 18009u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{
                return {{
                    "status": 200,
                    "body": "x",
                    "headers": {{"X-Custom": "test-value"}}
                }};
            }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    let custom = resp
        .headers()
        .get("x-custom")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(custom, "test-value");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_concurrent_requests() {
    require_server!();

    let port = 18010u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 200, "body": "ok"}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let handles: Vec<_> = (0..5)
        .map(|_| {
            let port = port;
            std::thread::spawn(move || {
                let client = make_client();
                client
                    .get(format!("http://127.0.0.1:{}/", port))
                    .send()
                    .map(|r| r.status().as_u16())
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(
        results
            .iter()
            .all(|r| r.as_ref().map(|&s| s == 200).unwrap_or(false)),
        "Not all concurrent requests returned 200: {:?}",
        results
    );
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_null_response() {
    require_server!();

    let port = 18011u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return null; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_query_params() {
    require_server!();

    let port = 18012u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 200, "body": req.query.name}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/?name=atlas&version=1", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "atlas");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_custom_status_201() {
    require_server!();

    let port = 18013u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 201, "body": "created"}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .post(format!("http://127.0.0.1:{}/", port))
        .body("")
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 201);
    assert_eq!(resp.text().unwrap(), "created");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_read_path_root() {
    require_server!();

    let port = 18014u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return {{"status": 200, "body": req.path}}; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "/");
}

#[test]
#[ignore = "requires server (ATLAS_TEST_SERVER=1)"]
#[cfg(feature = "http")]
fn test_serve_empty_string_response() {
    require_server!();

    let port = 18015u16;
    std::thread::spawn(move || {
        let atlas = Atlas::new_with_security(SecurityContext::allow_all());
        let _ = atlas.eval(&format!(
            r#"http.serve({}, fn(req) {{ return ""; }});"#,
            port
        ));
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    let client = make_client();
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .expect("Request failed");

    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(resp.text().unwrap(), "");
}
