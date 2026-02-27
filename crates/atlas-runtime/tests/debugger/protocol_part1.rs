//\! Protocol Serialization Tests (Part 1: sections 1-4)

use super::*;

// 1. Protocol serialization
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn proto_serialize_set_breakpoint() {
    let req = DebugRequest::SetBreakpoint { location: loc(5) };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_remove_breakpoint() {
    let req = DebugRequest::RemoveBreakpoint { id: 3 };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_continue() {
    let req = DebugRequest::Continue;
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_step_over() {
    let req = DebugRequest::StepOver;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepOver"));
}

#[test]
fn proto_serialize_step_into() {
    let req = DebugRequest::StepInto;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepInto"));
}

#[test]
fn proto_serialize_step_out() {
    let req = DebugRequest::StepOut;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepOut"));
}

#[test]
fn proto_serialize_get_variables() {
    let req = DebugRequest::GetVariables { frame_index: 2 };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_evaluate() {
    let req = DebugRequest::Evaluate {
        expression: "x + 1".to_string(),
        frame_index: 0,
    };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_paused_response() {
    let resp = DebugResponse::Paused {
        reason: PauseReason::Breakpoint { id: 1 },
        location: Some(loc(3)),
        ip: 42,
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_variables_response() {
    let resp = DebugResponse::Variables {
        frame_index: 0,
        variables: vec![Variable::new("x", "42", "Number")],
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_stack_trace_response() {
    let resp = DebugResponse::StackTrace {
        frames: vec![DebugStackFrame {
            index: 0,
            function_name: "<main>".to_string(),
            location: Some(loc(1)),
            stack_base: 0,
            local_count: 2,
        }],
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_error_response() {
    let resp = DebugResponse::Error {
        message: "unknown error".to_string(),
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_debug_event_paused() {
    let event = DebugEvent::Paused {
        reason: PauseReason::Step,
        location: Some(loc(2)),
        ip: 10,
    };
    let json = serialize_event(&event).unwrap();
    let back: DebugEvent = deserialize_event(&json).unwrap();
    assert_eq!(event, back);
}

#[test]
fn proto_serialize_debug_event_stopped() {
    let event = DebugEvent::Stopped {
        result: Some("42".to_string()),
        error: None,
    };
    let json = serialize_event(&event).unwrap();
    let back: DebugEvent = deserialize_event(&json).unwrap();
    assert_eq!(event, back);
}

#[test]
fn proto_source_location_display() {
    let loc = SourceLocation::new("main.atlas", 10, 5);
    assert_eq!(loc.to_string(), "main.atlas:10:5");
    let anon = SourceLocation::anonymous(3, 1);
    assert_eq!(anon.to_string(), "<anonymous>:3:1");
}

// ═════════════════════════════════════════════════════════════════════════════
// 2. Source mapping
// ═════════════════════════════════════════════════════════════════════════════

fn make_debug_spans(pairs: &[(usize, usize, usize)]) -> Vec<DebugSpan> {
    pairs
        .iter()
        .map(|&(off, s, e)| DebugSpan {
            instruction_offset: off,
            span: Span::new(s, e),
        })
        .collect()
}

#[test]
fn srcmap_compute_line_offsets_basic() {
    let src = "abc\ndef\nghi";
    let offsets = compute_line_offsets(src);
    assert_eq!(offsets[0], 0);
    assert_eq!(offsets[1], 4);
    assert_eq!(offsets[2], 8);
}

#[test]
fn srcmap_byte_offset_to_line_column_line1() {
    let src = "let x = 1;\nlet y = 2;";
    let offsets = compute_line_offsets(src);
    assert_eq!(byte_offset_to_line_column(0, &offsets), (1, 1));
    assert_eq!(byte_offset_to_line_column(4, &offsets), (1, 5));
}

#[test]
fn srcmap_byte_offset_to_line_column_line2() {
    let src = "let x = 1;\nlet y = 2;";
    let offsets = compute_line_offsets(src);
    let line2_start = 11; // after "let x = 1;\n"
    assert_eq!(byte_offset_to_line_column(line2_start, &offsets), (2, 1));
}

#[test]
fn srcmap_from_debug_spans_no_source_defaults() {
    let spans = make_debug_spans(&[(0, 0, 5), (5, 5, 10)]);
    let map = SourceMap::from_debug_spans(&spans, "test.atlas", None);
    let loc = map.location_for_offset(0).unwrap();
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 1);
}

#[test]
fn srcmap_from_debug_spans_with_source() {
    let src = "let x = 1;\nlet y = 2;\n";
    let spans = make_debug_spans(&[(0, 0, 10), (3, 11, 21)]);
    let map = SourceMap::from_debug_spans(&spans, "m.atlas", Some(src));
    let loc0 = map.location_for_offset(0).unwrap();
    let loc1 = map.location_for_offset(3).unwrap();
    assert_eq!(loc0.line, 1);
    assert_eq!(loc1.line, 2);
}

#[test]
fn srcmap_forward_lookup_exact() {
    let mut map = SourceMap::new();
    map.insert(10, SourceLocation::new("a.atlas", 3, 1));
    assert_eq!(map.location_for_offset(10).unwrap().line, 3);
}

#[test]
fn srcmap_forward_lookup_closest_preceding() {
    let mut map = SourceMap::new();
    map.insert(0, SourceLocation::new("a.atlas", 1, 1));
    map.insert(10, SourceLocation::new("a.atlas", 3, 1));
    assert_eq!(map.location_for_offset(5).unwrap().line, 1);
}

#[test]
fn srcmap_reverse_lookup_exact() {
    let mut map = SourceMap::new();
    map.insert(42, SourceLocation::new("a.atlas", 7, 3));
    assert_eq!(map.offset_for_location("a.atlas", 7, 3), Some(42));
}

#[test]
fn srcmap_offsets_for_line() {
    let mut map = SourceMap::new();
    map.insert(0, SourceLocation::new("a.atlas", 1, 1));
    map.insert(2, SourceLocation::new("a.atlas", 1, 5));
    map.insert(5, SourceLocation::new("a.atlas", 2, 1));
    let offsets = map.offsets_for_line("a.atlas", 1);
    assert_eq!(offsets, vec![0, 2]);
}

#[test]
fn srcmap_first_offset_for_line() {
    let mut map = SourceMap::new();
    map.insert(5, SourceLocation::new("a.atlas", 2, 1));
    map.insert(2, SourceLocation::new("a.atlas", 2, 5));
    assert_eq!(map.first_offset_for_line("a.atlas", 2), Some(2));
}

#[test]
fn srcmap_all_offsets_sorted() {
    let mut map = SourceMap::new();
    map.insert(10, SourceLocation::new("a.atlas", 1, 1));
    map.insert(3, SourceLocation::new("a.atlas", 1, 5));
    map.insert(7, SourceLocation::new("a.atlas", 2, 1));
    assert_eq!(map.all_offsets(), vec![3, 7, 10]);
}

#[test]
fn srcmap_empty_map_queries() {
    let map = SourceMap::new();
    assert!(map.is_empty());
    assert!(map.location_for_offset(0).is_none());
    assert_eq!(map.offset_for_location("a.atlas", 1, 1), None);
    assert!(map.offsets_for_line("a.atlas", 1).is_empty());
}

// ═════════════════════════════════════════════════════════════════════════════
// 3. Breakpoint management
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_add_breakpoint_assigns_sequential_ids() {
    let mut state = DebuggerState::new();
    let id1 = state.add_breakpoint(loc(1));
    let id2 = state.add_breakpoint(loc(2));
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn bp_unverified_by_default() {
    let mut state = DebuggerState::new();
    let id = state.add_breakpoint(loc(5));
    assert!(!state.get_breakpoint(id).unwrap().verified);
}

#[test]
fn bp_verify_binds_offset() {
    let mut state = DebuggerState::new();
    let id = state.add_breakpoint(loc(5));
    state.verify_breakpoint(id, 100);
    let bp = state.get_breakpoint(id).unwrap();
    assert!(bp.verified);
    assert_eq!(bp.instruction_offset, Some(100));
}

#[test]
fn bp_has_breakpoint_at_verified_offset() {
    let mut state = DebuggerState::new();
    let id = state.add_breakpoint(loc(3));
    state.verify_breakpoint(id, 50);
    assert!(state.has_breakpoint_at_offset(50));
    assert!(!state.has_breakpoint_at_offset(51));
}

#[test]
fn bp_unverified_does_not_match_offset() {
    let mut state = DebuggerState::new();
    state.add_breakpoint(loc(3));
    assert!(!state.has_breakpoint_at_offset(0));
}

#[test]
fn bp_remove_breakpoint() {
    let mut state = DebuggerState::new();
    let id = state.add_breakpoint(loc(5));
    state.remove_breakpoint(id);
    assert_eq!(state.breakpoint_count(), 0);
}

#[test]
fn bp_clear_all_breakpoints() {
    let mut state = DebuggerState::new();
    for line in 1..=5 {
        state.add_breakpoint(loc(line));
    }
    state.clear_breakpoints();
    assert_eq!(state.breakpoint_count(), 0);
}

#[test]
fn bp_session_set_returns_breakpoint_set() {
    let mut session = new_session("let x = 1;\n");
    match session.process_request(DebugRequest::SetBreakpoint { location: loc(1) }) {
        DebugResponse::BreakpointSet { breakpoint } => {
            assert_eq!(breakpoint.id, 1);
        }
        r => panic!("expected BreakpointSet, got {:?}", r),
    }
}

#[test]
fn bp_session_list_empty() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert!(breakpoints.is_empty()),
        r => panic!("{:?}", r),
    }
}

#[test]
fn bp_session_remove_existing() {
    let mut session = new_session("let x = 1;");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    match session.process_request(DebugRequest::RemoveBreakpoint { id: 1 }) {
        DebugResponse::BreakpointRemoved { id } => assert_eq!(id, 1),
        r => panic!("{:?}", r),
    }
}

#[test]
fn bp_session_remove_nonexistent_is_error() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::RemoveBreakpoint { id: 99 }) {
        DebugResponse::Error { .. } => {}
        r => panic!("expected Error, got {:?}", r),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 4. Step operations
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn step_into_mode_is_set() {
    let mut session = new_session("let x = 1;");
    session.process_request(DebugRequest::StepInto);
    assert_eq!(session.debug_state().step_mode, StepMode::Into);
}

#[test]
fn step_over_mode_is_set() {
    let mut session = new_session("let x = 1;");
    session.process_request(DebugRequest::StepOver);
    assert_eq!(session.debug_state().step_mode, StepMode::Over);
}

#[test]
fn step_out_mode_is_set() {
    let mut session = new_session("let x = 1;");
    session.process_request(DebugRequest::StepOut);
    assert_eq!(session.debug_state().step_mode, StepMode::Out);
}

#[test]
fn step_into_pauses_execution() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let mut session = new_session(source);
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        r => panic!("expected Paused, got {:?}", r),
    }
}
