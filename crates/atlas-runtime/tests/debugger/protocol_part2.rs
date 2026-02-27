//! Protocol Serialization Tests (Part 2: sections 5-8)

use super::*;

#[test]
fn step_state_over_logic() {
    let mut state = DebuggerState::new();
    state.set_step_mode(StepMode::Over, 2);
    assert!(state.should_pause_for_step(2)); // same depth → pause
    assert!(state.should_pause_for_step(1)); // shallower → pause
    assert!(!state.should_pause_for_step(3)); // deeper → keep going
}

#[test]
fn step_state_out_logic() {
    let mut state = DebuggerState::new();
    state.set_step_mode(StepMode::Out, 3);
    assert!(state.should_pause_for_step(2)); // returned
    assert!(!state.should_pause_for_step(3)); // still in same frame
}

#[test]
fn step_state_into_always_pauses() {
    let mut state = DebuggerState::new();
    state.set_step_mode(StepMode::Into, 1);
    assert!(state.should_pause_for_step(1));
    assert!(state.should_pause_for_step(5));
}

#[test]
fn step_mode_cleared_after_pause() {
    let mut state = DebuggerState::new();
    state.set_step_mode(StepMode::Into, 1);
    state.pause(PauseReason::Step, None, 5);
    assert_eq!(state.step_mode, StepMode::None);
}

// ═════════════════════════════════════════════════════════════════════════════
// 5. Variable inspection
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn vars_get_variables_response_has_correct_frame_index() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { frame_index, .. } => assert_eq!(frame_index, 0),
        r => panic!("{:?}", r),
    }
}

#[test]
fn vars_get_variables_nonexistent_frame() {
    let mut session = new_session("let x = 1;");
    // Should not panic, just return empty or global-only variables
    let resp = session.process_request(DebugRequest::GetVariables { frame_index: 99 });
    match resp {
        DebugResponse::Variables { .. } => {}
        r => panic!("expected Variables, got {:?}", r),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 6. Stack trace generation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_trace_has_main_frame() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert!(!frames.is_empty());
            assert_eq!(frames[0].function_name, "<main>");
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn stack_trace_innermost_frame_index_0() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => assert_eq!(frames[0].index, 0),
        r => panic!("{:?}", r),
    }
}

#[test]
fn stack_trace_frame_has_stack_base() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            // Main frame starts at stack base 0
            assert_eq!(frames[0].stack_base, 0);
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn stack_trace_serializable() {
    let mut session = new_session("let x = 1;");
    let resp = session.process_request(DebugRequest::GetStack);
    let json = serialize_response(&resp).unwrap();
    let back = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

// ═════════════════════════════════════════════════════════════════════════════
// 7. Expression evaluation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn eval_simple_arithmetic() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "2 + 2".to_string(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "number");
            assert!(value.contains('4'));
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn eval_boolean_expression() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "1 == 1".to_string(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "bool");
            assert!(value.contains("true"));
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn eval_string_concatenation() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: r#""foo" + "bar""#.to_string(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, .. } => {
            assert!(value.contains("foo"));
            assert!(value.contains("bar"));
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn eval_null_literal() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "null".to_string(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "null");
            assert!(value.contains("null"));
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn eval_invalid_syntax_does_not_panic() {
    let mut session = new_session("let x = 1;");
    // Should return EvalResult or Error, not panic
    let resp = session.process_request(DebugRequest::Evaluate {
        expression: "@#$%".to_string(),
        frame_index: 0,
    });
    match resp {
        DebugResponse::EvalResult { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected {:?}", r),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 8. Performance: no overhead when debugging disabled
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn perf_vm_without_debugger_runs_normally() {
    // Create a VM without debugging enabled – should produce correct results.
    let source = "let x = 10; let y = 20; let z = x + y;";
    let bc = compile(source);
    let mut vm = VM::new(bc);
    let sec = security();
    let result = vm.run(&sec);
    assert!(result.is_ok(), "VM should run without errors");
}

#[test]
fn perf_debugger_disabled_by_default() {
    let bc = compile("let x = 1;");
    let vm = VM::new(bc);
    // Debugger should be None (disabled) on a plain VM
    assert!(vm.debugger().is_none());
}

#[test]
fn perf_debugger_disabled_after_run() {
    // Running without debugging doesn't accidentally enable debugging.
    let bc = compile("let x = 1;");
    let mut vm = VM::new(bc);
    let sec = security();
    vm.run(&sec).unwrap();
    assert!(vm.debugger().is_none() || !vm.debugger().unwrap().is_enabled());
}

#[test]
fn perf_run_completes_correctly_multiple_operations() {
    let source = "let a = 5;\nlet b = 10;\nlet c = a + b;\nlet d = c * 2;";
    let bc = compile(source);
    let mut vm = VM::new(bc);
    let sec = security();
    let result = vm.run(&sec);
    assert!(result.is_ok());
}

// ═════════════════════════════════════════════════════════════════════════════
// Additional integration: debuggable session end-to-end
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_run_to_completion_no_breakpoints() {
    let source = "let x = 1 + 2;\nlet y = x * 3;";
    let mut session = new_session(source);
    let resp = session.run_until_pause(&security());
    // Should complete (either Paused with Step sentinel or similar)
    match resp {
        DebugResponse::Paused { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected {:?}", r),
    }
    assert!(session.is_stopped());
}

#[test]
fn e2e_get_location_returns_valid_ip() {
    let mut session = new_session("let x = 42;");
    match session.process_request(DebugRequest::GetLocation) {
        DebugResponse::Location { ip, .. } => {
            assert_eq!(ip, 0); // Before execution starts, IP is 0
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn e2e_breakpoint_id_is_stable() {
    let source = "let x = 1;\nlet y = 2;";
    let mut session = new_session(source);
    let id1 = match session.process_request(DebugRequest::SetBreakpoint { location: loc(1) }) {
        DebugResponse::BreakpointSet { breakpoint } => breakpoint.id,
        r => panic!("{:?}", r),
    };
    let id2 = match session.process_request(DebugRequest::SetBreakpoint { location: loc(2) }) {
        DebugResponse::BreakpointSet { breakpoint } => breakpoint.id,
        r => panic!("{:?}", r),
    };
    assert_ne!(id1, id2);
    // IDs are sequential
    assert_eq!(id2, id1 + 1);
}

#[test]
fn e2e_clear_breakpoints_empties_list() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let mut session = new_session(source);
    for line in 1..=3 {
        session.process_request(DebugRequest::SetBreakpoint {
            location: loc(line),
        });
    }
    session.process_request(DebugRequest::ClearBreakpoints);
    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert!(breakpoints.is_empty()),
        r => panic!("{:?}", r),
    }
}

#[test]
fn e2e_debug_state_initial_is_running() {
    let session = new_session("let x = 1;");
    assert!(session.debug_state().is_running());
}

#[test]
fn e2e_step_into_causes_paused_state() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let mut session = new_session(source);
    session.process_request(DebugRequest::StepInto);
    session.run_until_pause(&security());
    // After stepping, the state should be Paused or Stopped
    assert!(session.is_paused() || session.is_stopped());
}

// ═══════════════════════════════════════════════════════════════════════════════
