//! Breakpoint Manager and Step Tracker tests

use super::*;

// ══════════════════════════════════════════════════════════════════════════════
// Breakpoint Manager Tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_bp_manager_add_simple() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add(loc(1));
    assert_eq!(id, 1);
    assert_eq!(mgr.count(), 1);
}

#[test]
fn test_bp_manager_add_conditional_hit_count() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::HitCount(5));
    mgr.verify(id, 10);
    // Should skip until hit 5
    for _ in 0..4 {
        assert_eq!(mgr.check_offset(10), ShouldFire::Skip);
    }
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_bp_manager_add_conditional_hit_count_multiple() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::HitCountMultiple(3));
    mgr.verify(id, 10);
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // 1
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // 2
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause); // 3
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // 4
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // 5
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause); // 6
}

#[test]
fn test_bp_manager_log_point() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_log_point(loc(1), "value of x".to_string());
    mgr.verify(id, 10);
    // Log points return Skip (they log but don't pause)
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip);
    let logs = mgr.drain_log_output();
    assert_eq!(logs, vec!["value of x"]);
}

#[test]
fn test_bp_manager_log_point_accumulates() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_log_point(loc(1), "msg".to_string());
    mgr.verify(id, 10);
    mgr.check_offset(10);
    mgr.check_offset(10);
    let logs = mgr.drain_log_output();
    assert_eq!(logs.len(), 2);
}

#[test]
fn test_bp_manager_enable_disable() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add(loc(1));
    mgr.verify(id, 10);
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);

    mgr.disable(id);
    // Reset hit state by removing and re-adding
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip);

    mgr.enable(id);
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_bp_manager_remove_cleans_offset_index() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add(loc(1));
    mgr.verify(id, 10);
    assert!(mgr.has_breakpoint_at(10));
    mgr.remove(id);
    assert!(!mgr.has_breakpoint_at(10));
}

#[test]
fn test_bp_manager_clear_all() {
    let mut mgr = BreakpointManager::new();
    let id1 = mgr.add(loc(1));
    let id2 = mgr.add(loc(2));
    mgr.verify(id1, 10);
    mgr.verify(id2, 20);
    mgr.clear();
    assert_eq!(mgr.count(), 0);
    assert!(!mgr.has_breakpoint_at(10));
    assert!(!mgr.has_breakpoint_at(20));
}

#[test]
fn test_bp_manager_multiple_at_same_offset() {
    let mut mgr = BreakpointManager::new();
    let id1 = mgr.add(loc(1));
    let id2 = mgr.add_conditional(loc(1), BreakpointCondition::HitCount(3));
    mgr.verify(id1, 10);
    mgr.verify(id2, 10);
    // First bp (unconditional) should fire immediately
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_bp_manager_set_condition_after_creation() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add(loc(1));
    mgr.verify(id, 10);
    mgr.set_condition(id, BreakpointCondition::HitCount(2));
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // hit 1
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause); // hit 2
}

#[test]
fn test_bp_manager_reset_hit_counts() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::HitCount(2));
    mgr.verify(id, 10);
    mgr.check_offset(10); // hit 1
    mgr.reset_all_hit_counts();
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // hit 1 again (reset)
}

#[test]
fn test_bp_manager_expression_condition_always_passes() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::Expression("x > 0".into()));
    mgr.verify(id, 10);
    // Expression conditions pass through (caller must evaluate)
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_bp_manager_all_entries_sorted() {
    let mut mgr = BreakpointManager::new();
    mgr.add(loc(3));
    mgr.add(loc(1));
    mgr.add(loc(2));
    let ids: Vec<u32> = mgr.all_entries().iter().map(|e| e.breakpoint.id).collect();
    assert_eq!(ids, vec![1, 2, 3]);
}

#[test]
fn test_bp_manager_enabled_count() {
    let mut mgr = BreakpointManager::new();
    let id1 = mgr.add(loc(1));
    let _id2 = mgr.add(loc(2));
    assert_eq!(mgr.enabled_count(), 2);
    mgr.disable(id1);
    assert_eq!(mgr.enabled_count(), 1);
}

#[test]
fn test_bp_entry_unverified_skips() {
    let bp = Breakpoint::new(1, loc(1)); // unverified
    let mut entry = BreakpointEntry::new(bp);
    assert_eq!(entry.check_and_increment(), ShouldFire::Skip);
}

#[test]
fn test_bp_entry_disabled_skips() {
    let bp = Breakpoint::verified_at(1, loc(1), 10);
    let mut entry = BreakpointEntry::new(bp);
    entry.enabled = false;
    assert_eq!(entry.check_and_increment(), ShouldFire::Skip);
}

#[test]
fn test_bp_entry_is_log_point() {
    let bp = Breakpoint::new(1, loc(1));
    let entry = BreakpointEntry::log_point(bp, "msg".into());
    assert!(entry.is_log_point());
}

#[test]
fn test_bp_entry_not_log_point() {
    let bp = Breakpoint::new(1, loc(1));
    let entry = BreakpointEntry::new(bp);
    assert!(!entry.is_log_point());
}

// ══════════════════════════════════════════════════════════════════════════════
// Step Tracker Tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_step_tracker_initial() {
    let tracker = StepTracker::new();
    assert!(!tracker.is_stepping());
    assert!(tracker.active_request().is_none());
}

#[test]
fn test_step_tracker_begin_into() {
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::Into, 1, Some(&loc(1)));
    assert!(tracker.is_stepping());
    assert_eq!(tracker.active_request(), Some(&StepRequest::Into));
}

#[test]
fn test_step_tracker_begin_over() {
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::Over, 2, Some(&loc(3)));
    assert_eq!(tracker.start_depth(), 2);
}

#[test]
fn test_step_tracker_cancel() {
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::Into, 1, None);
    tracker.cancel();
    assert!(!tracker.is_stepping());
}

#[test]
fn test_step_tracker_run_to_offset() {
    let map = SourceMap::new();
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::RunToOffset(5), 1, None);
    assert!(tracker.should_pause(3, 1, &map).is_none());
    assert!(tracker.should_pause(5, 1, &map).is_some());
}

#[test]
fn test_step_tracker_instructions_counter() {
    let mut map = SourceMap::new();
    map.insert(0, SourceLocation::new("test.atlas", 1, 1));
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::Over, 1, Some(&loc(1)));
    tracker.should_pause(0, 2, &map); // deeper, same line
    tracker.should_pause(0, 2, &map);
    assert_eq!(tracker.instructions_executed(), 2);
}

#[test]
fn test_step_tracker_safety_limit() {
    let mut map = SourceMap::new();
    map.insert(0, SourceLocation::new("test.atlas", 1, 1));
    let mut tracker = StepTracker::new();
    tracker.set_max_instructions(3);
    tracker.begin_step(StepRequest::Over, 1, Some(&loc(1)));
    assert!(tracker.should_pause(0, 2, &map).is_none());
    assert!(tracker.should_pause(0, 2, &map).is_none());
    assert!(tracker.should_pause(0, 2, &map).is_none());
    assert!(tracker.should_pause(0, 2, &map).is_some()); // 4th call exceeds limit
}
