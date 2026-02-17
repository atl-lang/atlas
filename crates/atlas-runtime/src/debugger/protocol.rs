//! Debugger protocol - request/response/event types for communication.
//!
//! All types are serde-serializable for JSON transport, enabling a
//! debugger client (IDE, CLI, etc.) to communicate with the Atlas VM.

use serde::{Deserialize, Serialize};

// ── Primitive types ──────────────────────────────────────────────────────────

/// Unique identifier for a breakpoint.
pub type BreakpointId = u32;

// ── Source location ───────────────────────────────────────────────────────────

/// A position in a source file (1-based line and column).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Source file path (empty string for anonymous/REPL source).
    pub file: String,
    /// Line number (1-based).
    pub line: u32,
    /// Column number (1-based).
    pub column: u32,
}

impl SourceLocation {
    /// Create a source location with a named file.
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }

    /// Create a source location for anonymous/REPL source.
    pub fn anonymous(line: u32, column: u32) -> Self {
        Self {
            file: String::new(),
            line,
            column,
        }
    }

    /// Check if this is an anonymous (no-file) location.
    pub fn is_anonymous(&self) -> bool {
        self.file.is_empty()
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.file.is_empty() {
            write!(f, "<anonymous>:{}:{}", self.line, self.column)
        } else {
            write!(f, "{}:{}:{}", self.file, self.line, self.column)
        }
    }
}

// ── Breakpoint ────────────────────────────────────────────────────────────────

/// A registered breakpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Unique ID assigned by the debugger.
    pub id: BreakpointId,
    /// Requested source location.
    pub location: SourceLocation,
    /// Whether the breakpoint is bound to an actual instruction.
    pub verified: bool,
    /// Instruction offset this breakpoint is bound to (None if unverified).
    pub instruction_offset: Option<usize>,
}

impl Breakpoint {
    /// Create an unverified breakpoint at the requested location.
    pub fn new(id: BreakpointId, location: SourceLocation) -> Self {
        Self {
            id,
            location,
            verified: false,
            instruction_offset: None,
        }
    }

    /// Create a verified breakpoint already bound to an instruction offset.
    pub fn verified_at(id: BreakpointId, location: SourceLocation, offset: usize) -> Self {
        Self {
            id,
            location,
            verified: true,
            instruction_offset: Some(offset),
        }
    }
}

// ── Stack frame ───────────────────────────────────────────────────────────────

/// A frame in the call stack (for stack traces).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugStackFrame {
    /// Frame index: 0 = innermost (current), higher = outer.
    pub index: usize,
    /// Function name (or "<main>" for top-level code).
    pub function_name: String,
    /// Source location of the current instruction in this frame, if known.
    pub location: Option<SourceLocation>,
    /// Stack index where this frame's locals begin.
    pub stack_base: usize,
    /// Number of local variable slots in this frame.
    pub local_count: usize,
}

// ── Variable ──────────────────────────────────────────────────────────────────

/// A named variable with its value and type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    /// Variable name.
    pub name: String,
    /// Human-readable representation of the value.
    pub value: String,
    /// Atlas type name (e.g. "Number", "String", "Array").
    pub type_name: String,
}

impl Variable {
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
        type_name: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            type_name: type_name.into(),
        }
    }
}

// ── Pause reason ─────────────────────────────────────────────────────────────

/// Why execution was paused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PauseReason {
    /// A breakpoint was hit.
    Breakpoint {
        /// ID of the breakpoint that triggered the pause.
        id: BreakpointId,
    },
    /// A step operation completed (step-over, step-into, step-out).
    Step,
    /// Execution was paused manually (e.g. via Pause request).
    ManualPause,
    /// An exception/error caused execution to pause.
    Exception {
        /// Error message.
        message: String,
    },
}

// ── Requests ─────────────────────────────────────────────────────────────────

/// Requests sent from a debugger client to the Atlas VM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DebugRequest {
    // ── Breakpoint management ───────────────────────────────────────────────
    /// Register a breakpoint at a source location.
    SetBreakpoint { location: SourceLocation },
    /// Remove a previously registered breakpoint.
    RemoveBreakpoint { id: BreakpointId },
    /// List all registered breakpoints.
    ListBreakpoints,
    /// Remove all registered breakpoints.
    ClearBreakpoints,

    // ── Execution control ───────────────────────────────────────────────────
    /// Resume execution from a paused state.
    Continue,
    /// Step over the current source line (skip into calls).
    StepOver,
    /// Step into the next function call.
    StepInto,
    /// Step out of the current function.
    StepOut,
    /// Pause execution at the next instruction.
    Pause,

    // ── Inspection ─────────────────────────────────────────────────────────
    /// Get all variables visible in a stack frame.
    GetVariables {
        /// 0 = innermost frame.
        frame_index: usize,
    },
    /// Get the current call stack.
    GetStack,
    /// Evaluate an expression in the context of a stack frame.
    Evaluate {
        expression: String,
        /// Frame in which to evaluate the expression (0 = innermost).
        frame_index: usize,
    },
    /// Get the current execution location (instruction pointer + source).
    GetLocation,
}

// ── Responses ────────────────────────────────────────────────────────────────

/// Responses from the Atlas VM to a debugger client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DebugResponse {
    /// A breakpoint was successfully registered (possibly unverified).
    BreakpointSet { breakpoint: Breakpoint },
    /// A breakpoint was removed.
    BreakpointRemoved { id: BreakpointId },
    /// All registered breakpoints.
    Breakpoints { breakpoints: Vec<Breakpoint> },
    /// All breakpoints were cleared.
    BreakpointsCleared,
    /// Execution has been resumed.
    Resumed,
    /// Execution is paused.
    Paused {
        reason: PauseReason,
        location: Option<SourceLocation>,
        ip: usize,
    },
    /// Variables in the requested frame.
    Variables {
        frame_index: usize,
        variables: Vec<Variable>,
    },
    /// Current call stack.
    StackTrace { frames: Vec<DebugStackFrame> },
    /// Result of an expression evaluation.
    EvalResult { value: String, type_name: String },
    /// Current execution location.
    Location {
        location: Option<SourceLocation>,
        ip: usize,
    },
    /// An error occurred processing the request.
    Error { message: String },
}

impl DebugResponse {
    /// Convenience constructor for an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }
}

// ── Events ────────────────────────────────────────────────────────────────────

/// Asynchronous events emitted by the Atlas VM to debugger clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DebugEvent {
    /// VM has started execution.
    Started,
    /// VM is now paused (breakpoint, step, manual).
    Paused {
        reason: PauseReason,
        location: Option<SourceLocation>,
        ip: usize,
    },
    /// VM has resumed execution.
    Resumed,
    /// VM has stopped (completed normally or due to error).
    Stopped {
        /// Final value as a string (if completed normally).
        result: Option<String>,
        /// Error message (if stopped due to an error).
        error: Option<String>,
    },
    /// A breakpoint was bound to an actual instruction.
    BreakpointBound { id: BreakpointId, offset: usize },
    /// Output produced during execution.
    Output { text: String },
}

// ── Serialization helpers ─────────────────────────────────────────────────────

/// Serialize a debug request to JSON.
pub fn serialize_request(request: &DebugRequest) -> Result<String, serde_json::Error> {
    serde_json::to_string(request)
}

/// Deserialize a debug request from JSON.
pub fn deserialize_request(json: &str) -> Result<DebugRequest, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize a debug response to JSON.
pub fn serialize_response(response: &DebugResponse) -> Result<String, serde_json::Error> {
    serde_json::to_string(response)
}

/// Deserialize a debug response from JSON.
pub fn deserialize_response(json: &str) -> Result<DebugResponse, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize a debug event to JSON.
pub fn serialize_event(event: &DebugEvent) -> Result<String, serde_json::Error> {
    serde_json::to_string(event)
}

/// Deserialize a debug event from JSON.
pub fn deserialize_event(json: &str) -> Result<DebugEvent, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display_named() {
        let loc = SourceLocation::new("main.atlas", 10, 5);
        assert_eq!(loc.to_string(), "main.atlas:10:5");
    }

    #[test]
    fn test_source_location_display_anonymous() {
        let loc = SourceLocation::anonymous(3, 1);
        assert_eq!(loc.to_string(), "<anonymous>:3:1");
    }

    #[test]
    fn test_breakpoint_new_unverified() {
        let loc = SourceLocation::new("test.atlas", 5, 1);
        let bp = Breakpoint::new(1, loc.clone());
        assert_eq!(bp.id, 1);
        assert!(!bp.verified);
        assert!(bp.instruction_offset.is_none());
    }

    #[test]
    fn test_breakpoint_verified_at() {
        let loc = SourceLocation::new("test.atlas", 5, 1);
        let bp = Breakpoint::verified_at(2, loc, 42);
        assert!(bp.verified);
        assert_eq!(bp.instruction_offset, Some(42));
    }

    #[test]
    fn test_serialize_set_breakpoint_request() {
        let req = DebugRequest::SetBreakpoint {
            location: SourceLocation::new("main.atlas", 10, 1),
        };
        let json = serialize_request(&req).unwrap();
        assert!(json.contains("\"type\":\"SetBreakpoint\""));
        assert!(json.contains("\"line\":10"));
    }

    #[test]
    fn test_deserialize_set_breakpoint_request() {
        let json =
            r#"{"type":"SetBreakpoint","location":{"file":"main.atlas","line":10,"column":1}}"#;
        let req: DebugRequest = deserialize_request(json).unwrap();
        match req {
            DebugRequest::SetBreakpoint { location } => {
                assert_eq!(location.file, "main.atlas");
                assert_eq!(location.line, 10);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_serialize_continue_request() {
        let req = DebugRequest::Continue;
        let json = serialize_request(&req).unwrap();
        assert!(json.contains("\"type\":\"Continue\""));
    }

    #[test]
    fn test_serialize_step_over_request() {
        let req = DebugRequest::StepOver;
        let json = serialize_request(&req).unwrap();
        assert!(json.contains("\"type\":\"StepOver\""));
    }

    #[test]
    fn test_serialize_step_into_request() {
        let req = DebugRequest::StepInto;
        let json = serialize_request(&req).unwrap();
        assert!(json.contains("\"type\":\"StepInto\""));
    }

    #[test]
    fn test_serialize_step_out_request() {
        let req = DebugRequest::StepOut;
        let json = serialize_request(&req).unwrap();
        assert!(json.contains("\"type\":\"StepOut\""));
    }

    #[test]
    fn test_serialize_pause_request() {
        let req = DebugRequest::Pause;
        let json = serialize_request(&req).unwrap();
        assert!(json.contains("\"type\":\"Pause\""));
    }

    #[test]
    fn test_serialize_get_variables_request() {
        let req = DebugRequest::GetVariables { frame_index: 0 };
        let json = serialize_request(&req).unwrap();
        assert!(json.contains("\"type\":\"GetVariables\""));
        assert!(json.contains("\"frame_index\":0"));
    }

    #[test]
    fn test_serialize_evaluate_request() {
        let req = DebugRequest::Evaluate {
            expression: "x + 1".to_string(),
            frame_index: 0,
        };
        let json = serialize_request(&req).unwrap();
        assert!(json.contains("\"type\":\"Evaluate\""));
        assert!(json.contains("x + 1"));
    }

    #[test]
    fn test_serialize_paused_response() {
        let resp = DebugResponse::Paused {
            reason: PauseReason::Breakpoint { id: 1 },
            location: Some(SourceLocation::new("main.atlas", 5, 1)),
            ip: 42,
        };
        let json = serialize_response(&resp).unwrap();
        assert!(json.contains("\"type\":\"Paused\""));
        assert!(json.contains("\"ip\":42"));
    }

    #[test]
    fn test_serialize_variables_response() {
        let resp = DebugResponse::Variables {
            frame_index: 0,
            variables: vec![
                Variable::new("x", "42", "Number"),
                Variable::new("name", "\"hello\"", "String"),
            ],
        };
        let json = serialize_response(&resp).unwrap();
        let deserialized: DebugResponse = deserialize_response(&json).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_serialize_stack_trace_response() {
        let resp = DebugResponse::StackTrace {
            frames: vec![DebugStackFrame {
                index: 0,
                function_name: "main".to_string(),
                location: Some(SourceLocation::new("main.atlas", 5, 1)),
                stack_base: 0,
                local_count: 3,
            }],
        };
        let json = serialize_response(&resp).unwrap();
        let deserialized: DebugResponse = deserialize_response(&json).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_serialize_error_response() {
        let resp = DebugResponse::error("unknown breakpoint");
        let json = serialize_response(&resp).unwrap();
        assert!(json.contains("\"type\":\"Error\""));
        assert!(json.contains("unknown breakpoint"));
    }

    #[test]
    fn test_serialize_pause_reason_breakpoint() {
        let reason = PauseReason::Breakpoint { id: 3 };
        let json = serde_json::to_string(&reason).unwrap();
        assert!(json.contains("\"Breakpoint\"") || json.contains("\"kind\":\"Breakpoint\""));
    }

    #[test]
    fn test_serialize_pause_reason_step() {
        let reason = PauseReason::Step;
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: PauseReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }

    #[test]
    fn test_serialize_debug_event_paused() {
        let event = DebugEvent::Paused {
            reason: PauseReason::Step,
            location: Some(SourceLocation::anonymous(2, 1)),
            ip: 10,
        };
        let json = serialize_event(&event).unwrap();
        let deserialized: DebugEvent = deserialize_event(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_serialize_debug_event_stopped() {
        let event = DebugEvent::Stopped {
            result: Some("42".to_string()),
            error: None,
        };
        let json = serialize_event(&event).unwrap();
        let deserialized: DebugEvent = deserialize_event(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_roundtrip_all_requests() {
        let requests = vec![
            DebugRequest::SetBreakpoint {
                location: SourceLocation::new("a.atlas", 1, 1),
            },
            DebugRequest::RemoveBreakpoint { id: 1 },
            DebugRequest::ListBreakpoints,
            DebugRequest::ClearBreakpoints,
            DebugRequest::Continue,
            DebugRequest::StepOver,
            DebugRequest::StepInto,
            DebugRequest::StepOut,
            DebugRequest::Pause,
            DebugRequest::GetVariables { frame_index: 0 },
            DebugRequest::GetStack,
            DebugRequest::Evaluate {
                expression: "1+1".to_string(),
                frame_index: 0,
            },
            DebugRequest::GetLocation,
        ];
        for req in &requests {
            let json = serialize_request(req).unwrap();
            let back: DebugRequest = deserialize_request(&json).unwrap();
            assert_eq!(req, &back, "roundtrip failed for {:?}", req);
        }
    }
}
