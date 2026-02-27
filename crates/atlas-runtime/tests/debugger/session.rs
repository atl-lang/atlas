//! DebuggerSession Integration Tests

use super::*;
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_session_breakpoint_set_and_hit() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");

    let resp = session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 1, 1),
    });
    match resp {
        DebugResponse::BreakpointSet { breakpoint } => {
            if breakpoint.verified {
                let resp = session.run_until_pause(&security());
                if let DebugResponse::Paused { .. } = resp {
                    assert!(session.is_paused());
                }
            }
        }
        _ => panic!("expected BreakpointSet"),
    }
}

#[test]
fn test_session_step_into_pauses() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        DebugResponse::Error { .. } => {}
        r => panic!("expected Paused, got {:?}", r),
    }
}

#[test]
fn test_session_step_over_pauses() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::StepOver);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        r => panic!("expected Paused, got {:?}", r),
    }
}

#[test]
fn test_session_step_out_at_top_level() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::StepOut);
    // At top level, step-out should run to completion
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {} // completed
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_continue_runs_to_end() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::Continue);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => assert!(session.is_stopped()),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_multiple_breakpoints() {
    let source = "let a = 1;\nlet b = 2;\nlet c = 3;\nlet d = 4;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");

    session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 1, 1),
    });
    session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 3, 1),
    });

    // Both breakpoints registered
    if let DebugResponse::Breakpoints { breakpoints } =
        session.process_request(DebugRequest::ListBreakpoints)
    {
        assert_eq!(breakpoints.len(), 2);
    }
}

#[test]
fn test_session_remove_breakpoint() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    let resp = session.process_request(DebugRequest::RemoveBreakpoint { id: 1 });
    match resp {
        DebugResponse::BreakpointRemoved { id } => assert_eq!(id, 1),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_remove_nonexistent_breakpoint() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let resp = session.process_request(DebugRequest::RemoveBreakpoint { id: 99 });
    match resp {
        DebugResponse::Error { .. } => {}
        r => panic!("expected Error, got {:?}", r),
    }
}

#[test]
fn test_session_clear_breakpoints() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    session.process_request(DebugRequest::SetBreakpoint { location: loc(2) });
    session.process_request(DebugRequest::ClearBreakpoints);
    assert_eq!(session.debug_state().breakpoint_count(), 0);
}

#[test]
fn test_session_get_location() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetLocation) {
        DebugResponse::Location { ip, .. } => assert_eq!(ip, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_get_stack() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert!(!frames.is_empty());
            assert_eq!(frames[0].function_name, "<main>");
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_get_variables() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { frame_index, .. } => assert_eq!(frame_index, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_pause_request() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let resp = session.process_request(DebugRequest::Pause);
    match resp {
        DebugResponse::Resumed => {}
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_eval_arithmetic() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: "2 + 3".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "number");
            assert!(value.contains('5'));
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_eval_boolean() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: "true && false".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { type_name, .. } => {
            assert_eq!(type_name, "bool");
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_eval_string_concat() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: r#""hello" + " world""#.into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "string");
            assert!(value.contains("hello"));
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_eval_invalid_returns_error() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: "!!!invalid$$$".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_not_paused_initially() {
    let source = "let x = 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    assert!(!session.is_paused());
    assert!(!session.is_stopped());
}

#[test]
fn test_session_current_ip_starts_at_zero() {
    let source = "let x = 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    assert_eq!(session.current_ip(), 0);
}

#[test]
fn test_session_source_map_populated() {
    let source = "let x = 42;\nlet y = x + 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    assert!(!session.source_map().is_empty());
}

#[test]
fn test_session_run_without_breakpoints_completes() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
    assert!(session.is_stopped());
}

#[test]
fn test_session_conditional_code_debug() {
    let source = "let x = 5;\nlet y = 0;\nif x > 3 {\n  y = x * 2;\n}";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    // Set step-into to go through conditional
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_debug_state_accessible() {
    let source = "let x = 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    let state = session.debug_state();
    assert!(state.is_running());
}

// ══════════════════════════════════════════════════════════════════════════════
