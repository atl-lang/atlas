use super::*;
use atlas_runtime::async_runtime::FutureState;
use atlas_runtime::json_value::JsonValue;
use std::time::Duration;

// ============================================================================
// Watcher Tests
// ============================================================================

#[test]
fn test_fs_watch_emits_change_event() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("watch.txt");
    std_fs::write(&file_path, "seed").unwrap();
    let watcher = fs::watch(file_path.to_str().unwrap(), span()).unwrap();

    let state = atlas_runtime::async_runtime::block_on(async {
        let future_value = fs::watch_next(&watcher, span()).unwrap();
        let future = match future_value {
            Value::Future(f) => f.clone(),
            _ => panic!("Expected Future"),
        };

        std_fs::write(&file_path, "hello").unwrap();

        tokio::time::sleep(Duration::from_millis(500)).await;

        future.get_state()
    });

    match state {
        FutureState::Resolved(value) => match value {
            Value::JsonValue(json) => {
                let obj = json.as_object().expect("Expected watcher event object");
                let kind = obj
                    .get("kind")
                    .and_then(JsonValue::as_string)
                    .unwrap_or("unknown");
                assert_ne!(kind, "error");
                let paths = obj
                    .get("paths")
                    .and_then(JsonValue::as_array)
                    .cloned()
                    .unwrap_or_default();
                assert!(!paths.is_empty());
            }
            _ => panic!("Expected JsonValue event"),
        },
        FutureState::Rejected(err) => panic!("Watcher rejected: {}", err),
        FutureState::Pending => {}
    }
}
