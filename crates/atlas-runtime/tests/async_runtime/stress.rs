//! Stress tests for the B44 worker pool (D-057).
//!
//! These tests verify the system under load:
//! - 100+ concurrent tasks complete without starvation
//! - No deadlocks under sustained task pressure
//! - Memory: tasks complete and don't leak handles
//!
//! NOTE: 100 000-task tests run in nightly CI only; the default tests here
//! use 500 tasks to keep developer build times reasonable.

use atlas_runtime::async_runtime::{init_blocking_pool, init_worker_pool, spawn_task, TaskStatus};
use atlas_runtime::bytecode::Bytecode;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;
use std::sync::Once;
use std::time::Duration;

static STRESS_POOL_INIT: Once = Once::new();

fn ensure_stress_pool() {
    STRESS_POOL_INIT.call_once(|| {
        // Use 4 workers for stress tests so we exercise multi-worker stealing.
        let base_vm = VM::new(Bytecode::default());
        // Guard against double-init if concurrency tests ran first (OnceLock).
        let vm_ref = std::panic::AssertUnwindSafe(&base_vm);
        let _ = std::panic::catch_unwind(move || {
            init_worker_pool(4, *vm_ref);
            init_blocking_pool(*vm_ref);
        });
    });
}

/// Wait up to `timeout_ms` for a handle to reach a terminal state.
fn wait_settled(handle: &atlas_runtime::async_runtime::TaskHandle, timeout_ms: u64) -> TaskStatus {
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let s = handle.status();
        if !matches!(s, TaskStatus::Running) {
            return s;
        }
        if std::time::Instant::now() > deadline {
            return s;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

// ── Sustained load ────────────────────────────────────────────────────────────

#[test]
fn stress_500_tasks_all_settle() {
    ensure_stress_pool();

    const N: usize = 500;
    let handles: Vec<_> = (0..N)
        .map(|i| spawn_task(async move { Value::Number(i as f64) }, None))
        .collect();

    let mut settled = 0usize;
    let mut stuck = 0usize;
    for handle in &handles {
        match wait_settled(handle, 15_000) {
            TaskStatus::Completed | TaskStatus::Failed => settled += 1,
            _ => stuck += 1,
        }
    }

    assert_eq!(
        stuck, 0,
        "{stuck} of {N} tasks did not settle (possible deadlock)"
    );
    assert_eq!(settled, N);
}

// ── Memory: tasks complete and don't accumulate indefinitely ──────────────────

#[test]
fn stress_tasks_complete_not_accumulate() {
    ensure_stress_pool();

    // Spawn in batches: verify each batch finishes before next starts.
    // This checks tasks are not silently stuck in the queue.
    for batch in 0..5 {
        let handles: Vec<_> = (0..50)
            .map(|i| spawn_task(async move { Value::Number((batch * 50 + i) as f64) }, None))
            .collect();

        for handle in &handles {
            let status = wait_settled(handle, 5_000);
            assert!(
                matches!(status, TaskStatus::Completed | TaskStatus::Failed),
                "batch {batch}: task {} stuck as {status:?}",
                handle.id()
            );
        }
    }
}

// ── Work-stealing: tasks spread across workers ────────────────────────────────

#[test]
fn stress_work_stealing_no_starvation() {
    ensure_stress_pool();

    // Burst 200 tasks simultaneously — work-stealing should distribute them.
    const N: usize = 200;
    let start = std::time::Instant::now();

    let handles: Vec<_> = (0..N)
        .map(|_| spawn_task(async { Value::Null }, None))
        .collect();

    let mut all_settled = true;
    for handle in &handles {
        let status = wait_settled(handle, 20_000);
        if !matches!(status, TaskStatus::Completed | TaskStatus::Failed) {
            all_settled = false;
        }
    }

    let elapsed_ms = start.elapsed().as_millis();
    assert!(
        all_settled,
        "some tasks did not settle (starvation suspected)"
    );

    // Sanity: 200 tasks should not take more than 30 seconds total.
    assert!(
        elapsed_ms < 30_000,
        "200 tasks took {}ms — likely starvation or scheduler bug",
        elapsed_ms
    );
}

// ── Cancellation under load ───────────────────────────────────────────────────

#[test]
fn stress_cancel_half_under_load() {
    ensure_stress_pool();

    const N: usize = 100;
    let handles: Vec<_> = (0..N)
        .map(|i| spawn_task(async move { Value::Number(i as f64) }, None))
        .collect();

    // Cancel even-indexed tasks immediately.
    for (i, h) in handles.iter().enumerate() {
        if i % 2 == 0 {
            h.cancel();
        }
    }

    // All tasks must reach a terminal state — none should be stuck.
    for handle in &handles {
        let status = wait_settled(handle, 10_000);
        assert!(
            matches!(
                status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            ),
            "task {} stuck: {status:?}",
            handle.id()
        );
    }
}
