use super::*;
use pretty_assertions::assert_eq;

// ===========================================================================
// Helpers
// ===========================================================================

fn simple_add_bytecode() -> Bytecode {
    let mut bc = Bytecode::new();
    let idx_a = bc.add_constant(Value::Number(10.0));
    let idx_b = bc.add_constant(Value::Number(20.0));
    bc.emit(Opcode::Constant, atlas_runtime::span::Span::dummy());
    bc.emit_u16(idx_a);
    bc.emit(Opcode::Constant, atlas_runtime::span::Span::dummy());
    bc.emit_u16(idx_b);
    bc.emit(Opcode::Add, atlas_runtime::span::Span::dummy());
    bc.emit(Opcode::Halt, atlas_runtime::span::Span::dummy());
    bc
}

fn function_call_bytecode() -> Bytecode {
    // Defines a function that returns 42 and calls it once
    let mut bc = Bytecode::new();
    let span = atlas_runtime::span::Span::dummy();

    // function body starts after the Call + Halt in main
    // main: push FunctionRef, Call 0, Halt
    // fn body: Constant(42), Return
    let fn_body_offset = 10usize; // approximate — we'll place it exactly

    let func_ref = FunctionRef {
        name: "answer".to_string(),
        arity: 0,
        bytecode_offset: fn_body_offset,
        local_count: 1,
        param_ownership: vec![],
        param_names: vec![],
        return_ownership: None,
    };
    let func_idx = bc.add_constant(Value::Function(func_ref));
    let val_idx = bc.add_constant(Value::Number(42.0));

    // main: 0 - push func (3 bytes), 3 - Call u8 (2 bytes), 5 - Pop (1), 6 - Halt (1) = 7 bytes
    // fn body at offset 7
    bc.emit(Opcode::Constant, span);
    bc.emit_u16(func_idx);
    bc.emit(Opcode::Call, span);
    bc.emit_u8(0);
    bc.emit(Opcode::Halt, span);

    // Patch function offset
    let actual_fn_offset = bc.instructions.len();
    if let Value::Function(ref mut f) = bc.constants[func_idx as usize] {
        f.bytecode_offset = actual_fn_offset;
    }

    // fn body: push 42, Return
    bc.emit(Opcode::Constant, span);
    bc.emit_u16(val_idx);
    bc.emit(Opcode::Return, span);

    bc
}

// ===========================================================================
// Section 1: ProfileCollector unit tests
// ===========================================================================

#[test]
fn test_collector_empty_state() {
    let c = ProfileCollector::new();
    assert_eq!(c.total_instructions(), 0);
    assert!(c.instruction_counts().is_empty());
    assert_eq!(c.max_stack_depth(), 0);
    assert_eq!(c.function_calls(), 0);
}

#[test]
fn test_collector_counts_instructions() {
    let mut c = ProfileCollector::new();
    c.record_instruction(Opcode::Add, 0);
    c.record_instruction(Opcode::Add, 3);
    c.record_instruction(Opcode::Sub, 6);
    assert_eq!(c.total_instructions(), 3);
    assert_eq!(c.instruction_count(Opcode::Add), 2);
    assert_eq!(c.instruction_count(Opcode::Sub), 1);
}

#[test]
fn test_collector_tracks_location() {
    let mut c = ProfileCollector::new();
    for _ in 0..5 {
        c.record_instruction(Opcode::Loop, 100);
    }
    assert_eq!(c.location_counts()[&100], 5);
}

#[test]
fn test_collector_opcode_at_ip() {
    let mut c = ProfileCollector::new();
    c.record_instruction(Opcode::Mul, 42);
    assert_eq!(c.opcode_at(42), Some(Opcode::Mul));
}

#[test]
fn test_collector_max_stack_depth() {
    let mut c = ProfileCollector::new();
    c.update_frame_depth(1);
    c.update_frame_depth(8);
    c.update_frame_depth(3);
    assert_eq!(c.max_stack_depth(), 8);
}

#[test]
fn test_collector_function_calls() {
    let mut c = ProfileCollector::new();
    c.record_function_call("main");
    c.record_function_call("helper");
    c.record_function_call("helper");
    assert_eq!(c.function_calls(), 3);
    assert_eq!(c.function_call_counts()["helper"], 2);
}

#[test]
fn test_collector_reset() {
    let mut c = ProfileCollector::new();
    c.record_instruction(Opcode::Add, 0);
    c.update_frame_depth(5);
    c.record_function_call("f");
    c.reset();
    assert_eq!(c.total_instructions(), 0);
    assert_eq!(c.max_stack_depth(), 0);
    assert_eq!(c.function_calls(), 0);
}

#[test]
fn test_collector_top_opcodes_ordering() {
    let mut c = ProfileCollector::new();
    for _ in 0..30 {
        c.record_instruction(Opcode::Add, 0);
    }
    for _ in 0..10 {
        c.record_instruction(Opcode::Mul, 3);
    }
    let top = c.top_opcodes(2);
    assert_eq!(top[0].0, Opcode::Add);
    assert_eq!(top[1].0, Opcode::Mul);
}

#[test]
fn test_collector_top_locations() {
    let mut c = ProfileCollector::new();
    for _ in 0..100 {
        c.record_instruction(Opcode::Loop, 50);
    }
    for _ in 0..20 {
        c.record_instruction(Opcode::Add, 10);
    }
    let top = c.top_locations(1);
    assert_eq!(top[0].0, 50);
}

// ===========================================================================
// Section 2: HotspotDetector tests
// ===========================================================================

#[test]
fn test_hotspot_detector_default_threshold() {
    let d = HotspotDetector::new();
    assert!((d.threshold() - 1.0).abs() < 0.001);
}

#[test]
fn test_hotspot_detector_detects_loop() {
    let mut c = ProfileCollector::new();
    for _ in 0..50 {
        c.record_instruction(Opcode::Loop, 20);
    }
    for _ in 0..50 {
        c.record_instruction(Opcode::Add, 5);
    }
    let d = HotspotDetector::new();
    let hs = d.detect(&c);
    // Both are 50% — both should be detected
    assert_eq!(hs.len(), 2);
}

#[test]
fn test_hotspot_detector_threshold_filter() {
    let mut c = ProfileCollector::new();
    // 99 at ip=0, 1 at ip=1 (total=100 → ip=1 is 1.0% which equals threshold)
    for _ in 0..99 {
        c.record_instruction(Opcode::Add, 0);
    }
    c.record_instruction(Opcode::Mul, 1);
    let d = HotspotDetector::with_threshold(1.0);
    let hs = d.detect(&c);
    // ip=1 (1%) should be included (>= 1.0%)
    assert!(hs.iter().any(|h| h.ip == 1));
}

#[test]
fn test_hotspot_detector_sorts_by_count() {
    let mut c = ProfileCollector::new();
    for _ in 0..10 {
        c.record_instruction(Opcode::Add, 5);
    }
    for _ in 0..40 {
        c.record_instruction(Opcode::Loop, 10);
    }
    for _ in 0..50 {
        c.record_instruction(Opcode::Mul, 15);
    }
    let d = HotspotDetector::new();
    let hs = d.detect(&c);
    assert!(hs[0].count >= hs[1].count);
}

#[test]
fn test_hotspot_percentage_calculation() {
    let mut c = ProfileCollector::new();
    for _ in 0..1 {
        c.record_instruction(Opcode::Add, 0);
    }
    for _ in 0..9 {
        c.record_instruction(Opcode::Mul, 3);
    }
    let d = HotspotDetector::with_threshold(1.0);
    let hs = d.detect(&c);
    let mul_hs = hs.iter().find(|h| h.ip == 3).unwrap();
    assert!((mul_hs.percentage - 90.0).abs() < 0.1);
}

#[test]
fn test_hotspot_opcode_label() {
    let mut c = ProfileCollector::new();
    for _ in 0..100 {
        c.record_instruction(Opcode::GetLocal, 7);
    }
    let d = HotspotDetector::new();
    let hs = d.detect(&c);
    assert_eq!(hs[0].opcode, Some(Opcode::GetLocal));
}

// ===========================================================================
// Section 3: ProfileReport formatting tests
// ===========================================================================

fn make_full_report() -> ProfileReport {
    let mut p = Profiler::enabled();
    for _ in 0..100 {
        for i in 0..10usize {
            p.record_instruction_at(Opcode::Add, i * 3);
        }
    }
    p.update_frame_depth(4);
    p.update_value_stack_depth(12);
    p.record_function_call("compute");
    p.record_function_call("compute");
    p.generate_report(1.0)
}

#[test]
fn test_report_total_instructions() {
    let r = make_full_report();
    assert_eq!(r.total_instructions, 1000);
}

#[test]
fn test_report_max_stack_depth() {
    let r = make_full_report();
    assert_eq!(r.max_stack_depth, 4);
}

#[test]
fn test_report_function_calls() {
    let r = make_full_report();
    assert_eq!(r.function_calls, 2);
}

#[test]
fn test_report_top_opcodes_not_empty() {
    let r = make_full_report();
    assert!(!r.top_opcodes.is_empty());
}

#[test]
fn test_report_hotspots_detected() {
    let r = make_full_report();
    // Each of 10 locations gets exactly 10% — all above 1% threshold
    assert!(!r.hotspots.is_empty());
}

#[test]
fn test_report_summary_contains_count() {
    let r = make_full_report();
    let s = r.format_summary();
    assert!(s.contains("1000"), "summary: {}", s);
}

#[test]
fn test_report_detailed_contains_execution_section() {
    let r = make_full_report();
    let s = r.format_detailed();
    assert!(s.contains("Execution Summary"), "detailed: {}", s);
}

#[test]
fn test_report_detailed_contains_opcode_section() {
    let r = make_full_report();
    let s = r.format_detailed();
    assert!(s.contains("Top Opcodes"), "detailed: {}", s);
}

#[test]
fn test_report_detailed_contains_hotspot_section() {
    let r = make_full_report();
    let s = r.format_detailed();
    assert!(s.contains("Hotspot"), "detailed: {}", s);
}

#[test]
fn test_report_opcode_table_format() {
    let r = make_full_report();
    let s = r.format_opcode_table();
    assert!(s.contains("Add"), "opcode table: {}", s);
    assert!(s.contains("100.00%"), "opcode table: {}", s);
}

// ===========================================================================
// Section 4: Profiler struct integration
// ===========================================================================

#[test]
fn test_profiler_new_disabled() {
    let p = Profiler::new();
    assert!(!p.is_enabled());
}

#[test]
fn test_profiler_records_when_enabled() {
    let mut p = Profiler::enabled();
    p.record_instruction(Opcode::Add);
    p.record_instruction(Opcode::Add);
    assert_eq!(p.total_instructions(), 2);
}

#[test]
fn test_profiler_ignores_when_disabled() {
    let mut p = Profiler::new();
    p.record_instruction(Opcode::Add);
    assert_eq!(p.total_instructions(), 0);
}

#[test]
fn test_profiler_timing_records_elapsed() {
    let mut p = Profiler::enabled();
    p.start_timing();
    for i in 0..500usize {
        p.record_instruction_at(Opcode::Add, i % 50);
    }
    p.stop_timing();
    assert!(p.elapsed_secs().is_some());
    assert!(p.elapsed_secs().unwrap() >= 0.0);
}

#[test]
fn test_profiler_ips_is_positive() {
    let mut p = Profiler::enabled();
    p.start_timing();
    for i in 0..1000usize {
        p.record_instruction_at(Opcode::Mul, i % 100);
    }
    p.stop_timing();
    let r = p.generate_report(1.0);
    if let Some(ips) = r.ips {
        assert!(ips > 0.0);
    }
}

#[test]
fn test_profiler_hotspots_shorthand() {
    let mut p = Profiler::enabled();
    for _ in 0..100 {
        p.record_instruction_at(Opcode::Loop, 0);
    }
    assert!(!p.hotspots().is_empty());
}

#[test]
fn test_profiler_top_opcodes_shorthand() {
    let mut p = Profiler::enabled();
    for _ in 0..50 {
        p.record_instruction(Opcode::Add);
    }
    for _ in 0..20 {
        p.record_instruction(Opcode::Mul);
    }
    let top = p.top_opcodes(2);
    assert_eq!(top[0].opcode, Opcode::Add);
}

#[test]
fn test_profiler_reset() {
    let mut p = Profiler::enabled();
    p.record_instruction(Opcode::Add);
    p.reset();
    assert_eq!(p.total_instructions(), 0);
}

// ===========================================================================
// Section 5: VM integration tests
// ===========================================================================

#[test]
fn test_vm_with_profiling_enabled() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    let result = vm.run(&SecurityContext::allow_all()).unwrap();
    assert_eq!(result, Some(Value::Number(30.0)));

    let p = vm.profiler().unwrap();
    assert!(p.is_enabled());
    assert!(p.total_instructions() > 0);
}

#[test]
fn test_vm_instruction_count_accuracy() {
    // simple_add_bytecode: Constant, Constant, Add, Halt = 4 opcodes
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    assert_eq!(p.total_instructions(), 4);
    assert_eq!(p.instruction_count(Opcode::Constant), 2);
    assert_eq!(p.instruction_count(Opcode::Add), 1);
    assert_eq!(p.instruction_count(Opcode::Halt), 1);
}

#[test]
fn test_vm_profiling_not_enabled_by_default() {
    let bc = simple_add_bytecode();
    let mut vm = VM::new(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();
    assert!(vm.profiler().is_none());
}

#[test]
fn test_vm_profiling_records_stack_depth() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    // At some point during execution the value stack had at least 1 item
    assert!(p.collector().max_value_stack_depth() >= 1);
}

#[test]
fn test_vm_profiling_records_frame_depth() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    // main frame is always present, so at least depth 1
    assert!(p.max_stack_depth() >= 1);
}

#[test]
fn test_vm_profiling_function_call_tracking() {
    let bc = function_call_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    assert_eq!(p.function_calls(), 1);
    assert_eq!(p.collector().function_call_counts()["answer"], 1);
}

#[test]
fn test_vm_profiling_generates_report() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    let report = p.generate_report(1.0);
    assert_eq!(report.total_instructions, 4);
    assert!(!report.top_opcodes.is_empty());
}

#[test]
fn test_vm_profiling_timing_is_recorded() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    assert!(p.elapsed_secs().is_some(), "timing should be recorded");
}

#[test]
fn test_vm_enable_profiling_after_creation() {
    let bc = simple_add_bytecode();
    let mut vm = VM::new(bc);
    vm.enable_profiling();
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    assert!(p.total_instructions() > 0);
}

#[test]
fn test_vm_opcode_breakdown_correctness() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    let counts = p.instruction_counts();

    // Verify specific opcodes are counted
    assert_eq!(counts.get(&(Opcode::Add as u8)), Some(&1));
    assert_eq!(counts.get(&(Opcode::Constant as u8)), Some(&2));
}

#[test]
fn test_vm_report_ips_present_after_run() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let report = vm.profiler().unwrap().generate_report(1.0);
    // IPS should be populated since timing was recorded
    assert!(report.ips.is_some());
    assert!(report.ips.unwrap() > 0.0);
}

// NOTE: test block removed — required access to private function `is_none`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `len`

// === Migrated from src/profiler/report.rs ===
mod migrated_profiler_report {
    #![allow(unused_imports, dead_code, unused_variables, unused_mut)]
    use atlas_runtime::bytecode::Opcode;
    use atlas_runtime::profiler::ProfileReport;

    fn sample_report() -> ProfileReport {
        ProfileReport {
            total_instructions: 1000,
            elapsed_secs: Some(0.001),
            ips: Some(1_000_000.0),
            max_stack_depth: 4,
            max_value_stack_depth: 8,
            function_calls: 5,
            top_opcodes: vec![
                ("Add".to_string(), 400, 40.0),
                ("GetLocal".to_string(), 300, 30.0),
                ("Loop".to_string(), 300, 30.0),
            ],
            hotspots: vec![atlas_runtime::profiler::Hotspot {
                ip: 42,
                count: 300,
                percentage: 30.0,
                opcode: Some(atlas_runtime::bytecode::Opcode::Loop),
            }],
        }
    }

    #[test]
    fn test_format_summary_contains_instructions() {
        let r = sample_report();
        let s = r.format_summary();
        assert!(s.contains("1000 instructions"), "got: {}", s);
    }

    #[test]
    fn test_format_summary_contains_time() {
        let r = sample_report();
        let s = r.format_summary();
        assert!(s.contains("0.001s"), "got: {}", s);
    }

    #[test]
    fn test_format_summary_contains_ips() {
        let r = sample_report();
        let s = r.format_summary();
        assert!(s.contains("IPS"), "got: {}", s);
    }

    #[test]
    fn test_format_summary_no_timing() {
        let r = ProfileReport {
            total_instructions: 500,
            elapsed_secs: None,
            ips: None,
            max_stack_depth: 0,
            max_value_stack_depth: 0,
            function_calls: 0,
            top_opcodes: vec![],
            hotspots: vec![],
        };
        let s = r.format_summary();
        assert!(s.contains("500 instructions"));
        assert!(!s.contains("IPS"));
    }

    #[test]
    fn test_format_detailed_contains_summary() {
        let r = sample_report();
        let s = r.format_detailed();
        assert!(s.contains("1000"), "got: {}", s);
        assert!(s.contains("Execution Summary"), "got: {}", s);
    }

    #[test]
    fn test_format_detailed_contains_opcodes() {
        let r = sample_report();
        let s = r.format_detailed();
        assert!(s.contains("Add"), "got: {}", s);
        assert!(s.contains("40.00%"), "got: {}", s);
    }

    #[test]
    fn test_format_detailed_contains_hotspots() {
        let r = sample_report();
        let s = r.format_detailed();
        assert!(s.contains("42"), "got: {}", s);
        assert!(s.contains("Loop"), "got: {}", s);
    }

    #[test]
    fn test_format_detailed_no_hotspots_message() {
        let r = ProfileReport {
            total_instructions: 10,
            elapsed_secs: None,
            ips: None,
            max_stack_depth: 0,
            max_value_stack_depth: 0,
            function_calls: 0,
            top_opcodes: vec![],
            hotspots: vec![],
        };
        let s = r.format_detailed();
        assert!(s.contains("No hotspots"), "got: {}", s);
    }

    #[test]
    fn test_format_opcode_table_empty() {
        let r = ProfileReport {
            total_instructions: 0,
            elapsed_secs: None,
            ips: None,
            max_stack_depth: 0,
            max_value_stack_depth: 0,
            function_calls: 0,
            top_opcodes: vec![],
            hotspots: vec![],
        };
        let s = r.format_opcode_table();
        assert!(s.contains("No opcodes"), "got: {}", s);
    }

    #[test]
    fn test_format_opcode_table_with_data() {
        let r = sample_report();
        let s = r.format_opcode_table();
        assert!(s.contains("Add"));
        assert!(s.contains("400"));
        assert!(s.contains("40.00%"));
    }

    #[test]
    fn test_format_detailed_stack_depth() {
        let r = sample_report();
        let s = r.format_detailed();
        assert!(s.contains("4"), "expected max frame depth 4");
    }

    #[test]
    fn test_format_detailed_function_calls() {
        let r = sample_report();
        let s = r.format_detailed();
        assert!(s.contains("5"));
    }
}

// NOTE: test block removed — required access to private function `len`
