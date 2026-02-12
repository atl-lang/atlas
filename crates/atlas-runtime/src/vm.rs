//! Stack-based virtual machine
//!
//! Executes bytecode instructions with a value stack and call frames.
//! - Arithmetic operations check for NaN/Infinity
//! - Variables are stored in locals (stack) or globals (HashMap)
//! - Control flow uses jumps and loops

use crate::bytecode::{Bytecode, Opcode};
use crate::value::{RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Call frame for function calls
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Function name (for debugging)
    pub function_name: String,
    /// Return instruction pointer
    pub return_ip: usize,
    /// Base of local variables on stack
    pub stack_base: usize,
    /// Number of local variables
    pub local_count: usize,
}

/// Virtual machine state
pub struct VM {
    /// Value stack
    stack: Vec<Value>,
    /// Call frames (for function calls)
    frames: Vec<CallFrame>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Bytecode to execute
    bytecode: Bytecode,
    /// Instruction pointer
    ip: usize,
}

impl VM {
    /// Create a new VM with bytecode
    pub fn new(bytecode: Bytecode) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::new(),
            globals: HashMap::new(),
            bytecode,
            ip: 0,
        }
    }

    /// Execute the bytecode
    pub fn run(&mut self) -> Result<Option<Value>, RuntimeError> {
        loop {
            // Check if we've reached the end
            if self.ip >= self.bytecode.instructions.len() {
                break;
            }

            let opcode = self.read_opcode()?;

            match opcode {
                // ===== Constants =====
                Opcode::Constant => {
                    let index = self.read_u16() as usize;
                    if index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode);
                    }
                    let value = self.bytecode.constants[index].clone();
                    self.push(value);
                }
                Opcode::Null => self.push(Value::Null),
                Opcode::True => self.push(Value::Bool(true)),
                Opcode::False => self.push(Value::Bool(false)),

                // ===== Variables =====
                Opcode::GetLocal => {
                    let index = self.read_u16() as usize;
                    if index >= self.stack.len() {
                        return Err(RuntimeError::StackUnderflow);
                    }
                    let value = self.stack[index].clone();
                    self.push(value);
                }
                Opcode::SetLocal => {
                    let index = self.read_u16() as usize;
                    let value = self.peek(0).clone();
                    if index >= self.stack.len() {
                        // Need to extend stack
                        while self.stack.len() <= index {
                            self.stack.push(Value::Null);
                        }
                    }
                    self.stack[index] = value;
                }
                Opcode::GetGlobal => {
                    let name_index = self.read_u16() as usize;
                    if name_index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode);
                    }
                    let name = match &self.bytecode.constants[name_index] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => return Err(RuntimeError::TypeError(format!("Expected string constant for variable name"))),
                    };
                    let value = self
                        .globals
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()))?;
                    self.push(value);
                }
                Opcode::SetGlobal => {
                    let name_index = self.read_u16() as usize;
                    if name_index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode);
                    }
                    let name = match &self.bytecode.constants[name_index] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => return Err(RuntimeError::TypeError(format!("Expected string constant for variable name"))),
                    };
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                }

                // ===== Arithmetic =====
                Opcode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (Value::Number(x), Value::Number(y)) => {
                            let result = x + y;
                            if result.is_nan() || result.is_infinite() {
                                return Err(RuntimeError::InvalidNumericResult);
                            }
                            self.push(Value::Number(result));
                        }
                        (Value::String(x), Value::String(y)) => {
                            self.push(Value::String(Rc::new(format!("{}{}", x, y))));
                        }
                        _ => return Err(RuntimeError::TypeError(format!("Invalid operands for +"))),
                    }
                }
                Opcode::Sub => self.binary_numeric_op(|a, b| a - b)?,
                Opcode::Mul => self.binary_numeric_op(|a, b| a * b)?,
                Opcode::Div => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    let result = a / b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult);
                    }
                    self.push(Value::Number(result));
                }
                Opcode::Mod => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    let result = a % b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult);
                    }
                    self.push(Value::Number(result));
                }
                Opcode::Negate => {
                    let value = self.pop();
                    match value {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        _ => return Err(RuntimeError::TypeError(format!("Cannot negate non-number"))),
                    }
                }

                // ===== Comparison =====
                Opcode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Opcode::NotEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }
                Opcode::Less => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a < b));
                }
                Opcode::LessEqual => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a <= b));
                }
                Opcode::Greater => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a > b));
                }
                Opcode::GreaterEqual => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a >= b));
                }

                // ===== Logical =====
                Opcode::Not => {
                    let value = self.pop();
                    match value {
                        Value::Bool(b) => self.push(Value::Bool(!b)),
                        _ => return Err(RuntimeError::TypeError(format!("Cannot apply ! to non-boolean"))),
                    }
                }
                Opcode::And | Opcode::Or => {
                    // TODO: Short-circuit evaluation
                    return Err(RuntimeError::UnknownOpcode);
                }

                // ===== Control Flow =====
                Opcode::Jump => {
                    let offset = self.read_i16();
                    self.ip = (self.ip as isize + offset as isize) as usize;
                }
                Opcode::JumpIfFalse => {
                    let offset = self.read_i16();
                    let condition = self.pop();
                    if !condition.is_truthy() {
                        self.ip = (self.ip as isize + offset as isize) as usize;
                    }
                }
                Opcode::Loop => {
                    let offset = self.read_i16();
                    self.ip = (self.ip as isize - offset as isize) as usize;
                }

                // ===== Functions =====
                Opcode::Call | Opcode::Return => {
                    // TODO: Function calls in next phase
                    return Err(RuntimeError::UnknownOpcode);
                }

                // ===== Arrays =====
                Opcode::Array => {
                    let size = self.read_u16() as usize;
                    let mut elements = Vec::with_capacity(size);
                    for _ in 0..size {
                        elements.push(self.pop());
                    }
                    elements.reverse(); // Stack is LIFO, so reverse to get correct order
                    self.push(Value::Array(Rc::new(RefCell::new(elements))));
                }
                Opcode::GetIndex => {
                    let index = self.pop_number()?;
                    let array = self.pop();
                    match array {
                        Value::Array(arr) => {
                            if index.fract() != 0.0 || index < 0.0 {
                                return Err(RuntimeError::InvalidIndex);
                            }
                            let idx = index as usize;
                            let borrowed = arr.borrow();
                            if idx >= borrowed.len() {
                                return Err(RuntimeError::OutOfBounds);
                            }
                            self.push(borrowed[idx].clone());
                        }
                        _ => return Err(RuntimeError::TypeError(format!("Cannot index non-array"))),
                    }
                }
                Opcode::SetIndex => {
                    let value = self.pop();
                    let index = self.pop_number()?;
                    let array = self.pop();
                    match array {
                        Value::Array(arr) => {
                            if index.fract() != 0.0 || index < 0.0 {
                                return Err(RuntimeError::InvalidIndex);
                            }
                            let idx = index as usize;
                            let mut borrowed = arr.borrow_mut();
                            if idx >= borrowed.len() {
                                return Err(RuntimeError::OutOfBounds);
                            }
                            borrowed[idx] = value;
                        }
                        _ => return Err(RuntimeError::TypeError(format!("Cannot index non-array"))),
                    }
                }

                // ===== Stack Manipulation =====
                Opcode::Pop => {
                    // Don't pop if this is the last instruction before Halt
                    // Check if next instruction is Halt
                    if self.ip < self.bytecode.instructions.len()
                        && self.bytecode.instructions[self.ip] != Opcode::Halt as u8
                    {
                        self.pop();
                    }
                }
                Opcode::Dup => {
                    let value = self.peek(0).clone();
                    self.push(value);
                }

                // ===== Special =====
                Opcode::Halt => break,
            }
        }

        // Return top of stack if present
        Ok(if self.stack.is_empty() {
            None
        } else {
            Some(self.pop())
        })
    }

    // ===== Helper Methods =====

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Stack underflow")
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn pop_number(&mut self) -> Result<f64, RuntimeError> {
        match self.pop() {
            Value::Number(n) => Ok(n),
            _ => Err(RuntimeError::TypeError(format!("Expected number"))),
        }
    }

    fn binary_numeric_op<F>(&mut self, op: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        let result = op(a, b);
        if result.is_nan() || result.is_infinite() {
            return Err(RuntimeError::InvalidNumericResult);
        }
        self.push(Value::Number(result));
        Ok(())
    }

    fn read_opcode(&mut self) -> Result<Opcode, RuntimeError> {
        if self.ip >= self.bytecode.instructions.len() {
            return Err(RuntimeError::UnknownOpcode);
        }
        let byte = self.bytecode.instructions[self.ip];
        self.ip += 1;
        Opcode::try_from(byte).map_err(|_| RuntimeError::UnknownOpcode)
    }

    fn read_u8(&mut self) -> u8 {
        let byte = self.bytecode.instructions[self.ip];
        self.ip += 1;
        byte
    }

    fn read_u16(&mut self) -> u16 {
        let hi = self.bytecode.instructions[self.ip] as u16;
        let lo = self.bytecode.instructions[self.ip + 1] as u16;
        self.ip += 2;
        (hi << 8) | lo
    }

    fn read_i16(&mut self) -> i16 {
        self.read_u16() as i16
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Bytecode::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn execute_source(source: &str) -> Result<Option<Value>, RuntimeError> {
        // Compile source to bytecode
        let mut lexer = Lexer::new(source.to_string());
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (program, _) = parser.parse();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("Compilation failed");

        // Execute on VM
        let mut vm = VM::new(bytecode);
        vm.run()
    }

    #[test]
    fn test_vm_number_literal() {
        let result = execute_source("42;").unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_arithmetic() {
        let result = execute_source("2 + 3;").unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));

        let result = execute_source("10 - 4;").unwrap();
        assert_eq!(result, Some(Value::Number(6.0)));

        let result = execute_source("3 * 4;").unwrap();
        assert_eq!(result, Some(Value::Number(12.0)));

        let result = execute_source("15 / 3;").unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_comparison() {
        let result = execute_source("1 < 2;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = execute_source("5 > 10;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));

        let result = execute_source("3 == 3;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_vm_global_variable() {
        let result = execute_source("let x = 42; x;").unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_string_concat() {
        let result = execute_source("\"hello\" + \" world\";").unwrap();
        if let Some(Value::String(s)) = result {
            assert_eq!(s.as_ref(), "hello world");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_vm_array_literal() {
        let result = execute_source("[1, 2, 3];").unwrap();
        if let Some(Value::Array(arr)) = result {
            let borrowed = arr.borrow();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[1], Value::Number(2.0));
            assert_eq!(borrowed[2], Value::Number(3.0));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_vm_array_index() {
        let result = execute_source("let arr = [10, 20, 30]; arr[1];").unwrap();
        assert_eq!(result, Some(Value::Number(20.0)));
    }

    #[test]
    fn test_vm_division_by_zero() {
        let result = execute_source("10 / 0;");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero));
    }

    #[test]
    fn test_vm_bool_literals() {
        let result = execute_source("true;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = execute_source("false;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));
    }

    #[test]
    fn test_vm_null_literal() {
        let result = execute_source("null;").unwrap();
        assert_eq!(result, Some(Value::Null));
    }

    #[test]
    fn test_vm_unary_negate() {
        let result = execute_source("-42;").unwrap();
        assert_eq!(result, Some(Value::Number(-42.0)));
    }

    #[test]
    fn test_vm_logical_not() {
        let result = execute_source("!true;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));

        let result = execute_source("!false;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
    }
}
