//! Interpreter Debugger Tests (Parity with VM debugger)

use super::*;
use atlas_runtime::interpreter::debugger::InterpreterDebuggerSession;

fn interp_session(source: &str) -> InterpreterDebuggerSession {
    InterpreterDebuggerSession::new(source, "test.atlas")
}

// ── Interpreter Debugger: Session creation ────────────────────────────────────

#[test]
fn interp_session_creation() {
    let session = interp_session("let x = 1;");
    assert!(!session.is_paused());
    assert!(!session.is_stopped());
}

#[test]
fn interp_session_initial_frame_depth() {
    let session = interp_session("let x = 1;");
    assert_eq!(session.frame_depth(), 1); // Main frame
}

// ── Interpreter Debugger: Breakpoint management ───────────────────────────────

#[test]
fn interp_set_breakpoint_returns_id() {
    let mut session = interp_session("let x = 1;\nlet y = 2;");
    match session.process_request(DebugRequest::SetBreakpoint { location: loc(1) }) {
        DebugResponse::BreakpointSet { breakpoint } => assert_eq!(breakpoint.id, 1),
        r => panic!("expected BreakpointSet, got {:?}", r),
    }
}

#[test]
fn interp_set_multiple_breakpoints() {
    let mut session = interp_session("let a = 1;\nlet b = 2;\nlet c = 3;");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    session.process_request(DebugRequest::SetBreakpoint { location: loc(2) });
    session.process_request(DebugRequest::SetBreakpoint { location: loc(3) });

    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert_eq!(breakpoints.len(), 3),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_remove_breakpoint() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    match session.process_request(DebugRequest::RemoveBreakpoint { id: 1 }) {
        DebugResponse::BreakpointRemoved { id } => assert_eq!(id, 1),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_remove_nonexistent_breakpoint_error() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::RemoveBreakpoint { id: 99 }) {
        DebugResponse::Error { .. } => {}
        r => panic!("expected Error, got {:?}", r),
    }
}

#[test]
fn interp_clear_breakpoints() {
    let mut session = interp_session("let x = 1;\nlet y = 2;");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    session.process_request(DebugRequest::SetBreakpoint { location: loc(2) });
    session.process_request(DebugRequest::ClearBreakpoints);
    assert_eq!(session.debug_state().breakpoint_count(), 0);
}

#[test]
fn interp_list_breakpoints_empty() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert!(breakpoints.is_empty()),
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Step modes ──────────────────────────────────────────

#[test]
fn interp_step_into_mode_set() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::StepInto);
    assert_eq!(session.debug_state().step_mode, StepMode::Into);
}

#[test]
fn interp_step_over_mode_set() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::StepOver);
    assert_eq!(session.debug_state().step_mode, StepMode::Over);
}

#[test]
fn interp_step_out_mode_set() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::StepOut);
    assert_eq!(session.debug_state().step_mode, StepMode::Out);
}

#[test]
fn interp_step_into_pauses_execution() {
    let source = "let x = 1;\nlet y = 2;";
    let mut session = interp_session(source);
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { reason, .. } => {
            assert_eq!(reason, PauseReason::Step);
        }
        r => panic!("expected Paused, got {:?}", r),
    }
}

#[test]
fn interp_continue_runs_to_end() {
    let source = "let x = 1;\nlet y = 2;";
    let mut session = interp_session(source);
    session.process_request(DebugRequest::Continue);
    session.run_until_pause(&security());
    assert!(session.is_stopped());
}

// ── Interpreter Debugger: Stack trace ─────────────────────────────────────────

#[test]
fn interp_get_stack_has_main() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert!(!frames.is_empty());
            assert_eq!(frames[0].function_name, "<main>");
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_stack_frame_index_zero() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert_eq!(frames[0].index, 0);
        }
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Variable inspection ─────────────────────────────────

#[test]
fn interp_get_variables_frame_0() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { frame_index, .. } => assert_eq!(frame_index, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_get_variables_includes_globals() {
    let source = "let x = 42;";
    let mut session = interp_session(source);
    // Run to completion so variable is defined
    session.process_request(DebugRequest::Continue);
    session.run_until_pause(&security());

    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { variables, .. } => {
            // Should have builtins at minimum
            assert!(!variables.is_empty());
        }
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Expression evaluation ───────────────────────────────

#[test]
fn interp_eval_simple_arithmetic() {
    let mut session = interp_session("let x = 1;");
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
fn interp_eval_string_expression() {
    let mut session = interp_session("let x = 1;");
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
fn interp_eval_boolean_expression() {
    let mut session = interp_session("let x = 1;");
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
fn interp_eval_null_literal() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "null".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "null");
            assert!(value.contains("null"));
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_eval_invalid_syntax_error() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "!!!bad$$$".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Location ────────────────────────────────────────────

#[test]
fn interp_get_location_initial() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::GetLocation) {
        DebugResponse::Location { ip, .. } => assert_eq!(ip, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Pause request ───────────────────────────────────────

#[test]
fn interp_pause_request_sets_step_mode() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::Pause);
    // Pause sets step-into mode
    assert_eq!(session.debug_state().step_mode, StepMode::Into);
}

// ── Interpreter Debugger: End-to-end ──────────────────────────────────────────

#[test]
fn interp_e2e_run_to_completion() {
    let source = "let x = 1 + 2;\nlet y = x * 3;";
    let mut session = interp_session(source);
    session.run_until_pause(&security());
    assert!(session.is_stopped());
}

#[test]
fn interp_e2e_breakpoint_ids_sequential() {
    let mut session = interp_session("let x = 1;\nlet y = 2;");
    let id1 = match session.process_request(DebugRequest::SetBreakpoint { location: loc(1) }) {
        DebugResponse::BreakpointSet { breakpoint } => breakpoint.id,
        r => panic!("{:?}", r),
    };
    let id2 = match session.process_request(DebugRequest::SetBreakpoint { location: loc(2) }) {
        DebugResponse::BreakpointSet { breakpoint } => breakpoint.id,
        r => panic!("{:?}", r),
    };
    assert_ne!(id1, id2);
    assert_eq!(id2, id1 + 1);
}

#[test]
fn interp_e2e_conditional_code() {
    let source = "let x = 5;\nif (x > 3) {\n  let y = x * 2;\n}";
    let mut session = interp_session(source);
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_debug_state_accessible() {
    let session = interp_session("let x = 1;");
    let state = session.debug_state();
    assert!(state.is_running());
}

// ── Interpreter-VM Parity Tests ───────────────────────────────────────────────

#[test]
fn parity_both_support_set_breakpoint() {
    let source = "let x = 1;\nlet y = 2;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    let interp_resp =
        interp_session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });

    match (vm_resp, interp_resp) {
        (DebugResponse::BreakpointSet { .. }, DebugResponse::BreakpointSet { .. }) => {}
        r => panic!("expected both BreakpointSet, got {:?}", r),
    }
}

#[test]
fn parity_both_support_list_breakpoints() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::ListBreakpoints);
    let interp_resp = interp_session.process_request(DebugRequest::ListBreakpoints);

    match (vm_resp, interp_resp) {
        (DebugResponse::Breakpoints { .. }, DebugResponse::Breakpoints { .. }) => {}
        r => panic!("expected both Breakpoints, got {:?}", r),
    }
}

#[test]
fn parity_both_support_step_into() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    vm_session.process_request(DebugRequest::StepInto);
    interp_session.process_request(DebugRequest::StepInto);

    assert_eq!(vm_session.debug_state().step_mode, StepMode::Into);
    assert_eq!(interp_session.debug_state().step_mode, StepMode::Into);
}

#[test]
fn parity_both_support_step_over() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    vm_session.process_request(DebugRequest::StepOver);
    interp_session.process_request(DebugRequest::StepOver);

    assert_eq!(vm_session.debug_state().step_mode, StepMode::Over);
    assert_eq!(interp_session.debug_state().step_mode, StepMode::Over);
}

#[test]
fn parity_both_support_step_out() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    vm_session.process_request(DebugRequest::StepOut);
    interp_session.process_request(DebugRequest::StepOut);

    assert_eq!(vm_session.debug_state().step_mode, StepMode::Out);
    assert_eq!(interp_session.debug_state().step_mode, StepMode::Out);
}

#[test]
fn parity_both_support_get_stack() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::GetStack);
    let interp_resp = interp_session.process_request(DebugRequest::GetStack);

    match (vm_resp, interp_resp) {
        (DebugResponse::StackTrace { frames: f1 }, DebugResponse::StackTrace { frames: f2 }) => {
            assert!(!f1.is_empty());
            assert!(!f2.is_empty());
            // Both should have <main> frame
            assert_eq!(f1[0].function_name, "<main>");
            assert_eq!(f2[0].function_name, "<main>");
        }
        r => panic!("expected both StackTrace, got {:?}", r),
    }
}

#[test]
fn parity_both_support_get_variables() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::GetVariables { frame_index: 0 });
    let interp_resp = interp_session.process_request(DebugRequest::GetVariables { frame_index: 0 });

    match (vm_resp, interp_resp) {
        (
            DebugResponse::Variables {
                frame_index: fi1, ..
            },
            DebugResponse::Variables {
                frame_index: fi2, ..
            },
        ) => {
            assert_eq!(fi1, 0);
            assert_eq!(fi2, 0);
        }
        r => panic!("expected both Variables, got {:?}", r),
    }
}

#[test]
fn parity_both_support_evaluate() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::Evaluate {
        expression: "1 + 2".into(),
        frame_index: 0,
    });
    let interp_resp = interp_session.process_request(DebugRequest::Evaluate {
        expression: "1 + 2".into(),
        frame_index: 0,
    });

    match (vm_resp, interp_resp) {
        (
            DebugResponse::EvalResult {
                value: v1,
                type_name: t1,
            },
            DebugResponse::EvalResult {
                value: v2,
                type_name: t2,
            },
        ) => {
            assert_eq!(t1, "number");
            assert_eq!(t2, "number");
            assert!(v1.contains('3'));
            assert!(v2.contains('3'));
        }
        r => panic!("expected both EvalResult, got {:?}", r),
    }
}

#[test]
fn parity_both_support_continue() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::Continue);
    let interp_resp = interp_session.process_request(DebugRequest::Continue);

    match (vm_resp, interp_resp) {
        (DebugResponse::Resumed, DebugResponse::Resumed) => {}
        r => panic!("expected both Resumed, got {:?}", r),
    }
}
