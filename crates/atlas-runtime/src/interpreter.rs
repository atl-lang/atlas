//! AST interpreter (tree-walking)
//!
//! Direct AST evaluation with environment-based variable storage.
//! Supports:
//! - Expression evaluation (literals, binary/unary ops, calls, indexing)
//! - Statement execution (declarations, assignments, control flow)
//! - Function calls and stack frames
//! - Block scoping with shadowing

use crate::ast::{
    Assign, AssignTarget, BinaryExpr, BinaryOp, Block, CallExpr, Expr, ForStmt,
    IfStmt, IndexExpr, Item, Literal, Param, Program, ReturnStmt, Stmt, UnaryExpr, UnaryOp,
    VarDecl, WhileStmt,
};
use crate::value::{FunctionRef, RuntimeError, Value};
use std::collections::HashMap;

/// Control flow signal for handling break, continue, and return
#[derive(Debug, Clone, PartialEq)]
enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}

/// User-defined function
#[derive(Debug, Clone)]
struct UserFunction {
    name: String,
    params: Vec<Param>,
    body: Block,
}

/// Interpreter state
pub struct Interpreter {
    /// Global variables
    globals: HashMap<String, Value>,
    /// Local scopes (stack of environments)
    locals: Vec<HashMap<String, Value>>,
    /// User-defined functions
    functions: HashMap<String, UserFunction>,
    /// Current control flow state
    control_flow: ControlFlow,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            functions: HashMap::new(),
            control_flow: ControlFlow::None,
        }
    }

    /// Evaluate a program
    pub fn eval(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Null;

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    // Store user-defined function
                    self.functions.insert(
                        func.name.name.clone(),
                        UserFunction {
                            name: func.name.name.clone(),
                            params: func.params.clone(),
                            body: func.body.clone(),
                        },
                    );

                    // Also store as a value for reference
                    let func_value = Value::Function(FunctionRef {
                        name: func.name.name.clone(),
                        arity: func.params.len(),
                        bytecode_offset: 0, // Not used in interpreter
                    });
                    self.globals.insert(func.name.name.clone(), func_value);
                }
                Item::Statement(stmt) => {
                    last_value = self.eval_statement(stmt)?;

                    // Check for early return at top level
                    if let ControlFlow::Return(val) = &self.control_flow {
                        last_value = val.clone();
                        self.control_flow = ControlFlow::None;
                        break;
                    }
                }
            }
        }

        Ok(last_value)
    }

    /// Execute a statement
    fn eval_statement(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::VarDecl(var) => self.eval_var_decl(var),
            Stmt::Assign(assign) => self.eval_assign(assign),
            Stmt::If(if_stmt) => self.eval_if(if_stmt),
            Stmt::While(while_stmt) => self.eval_while(while_stmt),
            Stmt::For(for_stmt) => self.eval_for(for_stmt),
            Stmt::Return(return_stmt) => self.eval_return(return_stmt),
            Stmt::Break(_) => {
                self.control_flow = ControlFlow::Break;
                Ok(Value::Null)
            }
            Stmt::Continue(_) => {
                self.control_flow = ControlFlow::Continue;
                Ok(Value::Null)
            }
            Stmt::Expr(expr_stmt) => self.eval_expr(&expr_stmt.expr),
        }
    }

    /// Evaluate a variable declaration
    fn eval_var_decl(&mut self, var: &VarDecl) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(&var.init)?;
        let scope = self.locals.last_mut().unwrap();
        scope.insert(var.name.name.clone(), value);
        Ok(Value::Null)
    }

    /// Evaluate an assignment
    fn eval_assign(&mut self, assign: &Assign) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(&assign.value)?;

        match &assign.target {
            AssignTarget::Name(id) => {
                self.set_variable(&id.name, value)?;
            }
            AssignTarget::Index { target, index, .. } => {
                let arr_val = self.eval_expr(target)?;
                let idx_val = self.eval_expr(index)?;
                self.set_array_element(arr_val, idx_val, value)?;
            }
        }

        Ok(Value::Null)
    }

    /// Evaluate an if statement
    fn eval_if(&mut self, if_stmt: &IfStmt) -> Result<Value, RuntimeError> {
        let condition = self.eval_expr(&if_stmt.cond)?;

        if condition.is_truthy() {
            self.eval_block(&if_stmt.then_block)
        } else if let Some(else_block) = &if_stmt.else_block {
            self.eval_block(else_block)
        } else {
            Ok(Value::Null)
        }
    }

    /// Evaluate a while loop
    fn eval_while(&mut self, while_stmt: &WhileStmt) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Null;

        loop {
            let condition = self.eval_expr(&while_stmt.cond)?;

            if !condition.is_truthy() {
                break;
            }

            last_value = self.eval_block(&while_stmt.body)?;

            match self.control_flow {
                ControlFlow::Break => {
                    self.control_flow = ControlFlow::None;
                    break;
                }
                ControlFlow::Continue => {
                    self.control_flow = ControlFlow::None;
                    continue;
                }
                ControlFlow::Return(_) => {
                    // Propagate return up
                    break;
                }
                ControlFlow::None => {}
            }
        }

        Ok(last_value)
    }

    /// Evaluate a for loop
    fn eval_for(&mut self, for_stmt: &ForStmt) -> Result<Value, RuntimeError> {
        // Push new scope for loop variable
        self.push_scope();

        // Initialize loop variable
        self.eval_statement(&for_stmt.init)?;

        let mut last_value = Value::Null;

        loop {
            // Check condition
            let cond_val = self.eval_expr(&for_stmt.cond)?;
            if !cond_val.is_truthy() {
                break;
            }

            // Execute body
            last_value = self.eval_block(&for_stmt.body)?;

            match self.control_flow {
                ControlFlow::Break => {
                    self.control_flow = ControlFlow::None;
                    break;
                }
                ControlFlow::Continue => {
                    self.control_flow = ControlFlow::None;
                    // Continue to step
                }
                ControlFlow::Return(_) => {
                    // Propagate return up
                    break;
                }
                ControlFlow::None => {}
            }

            // Execute step
            self.eval_statement(&for_stmt.step)?;
        }

        self.pop_scope();
        Ok(last_value)
    }

    /// Evaluate a return statement
    fn eval_return(&mut self, return_stmt: &ReturnStmt) -> Result<Value, RuntimeError> {
        let value = if let Some(expr) = &return_stmt.value {
            self.eval_expr(expr)?
        } else {
            Value::Null
        };

        self.control_flow = ControlFlow::Return(value.clone());
        Ok(value)
    }

    /// Evaluate a block
    fn eval_block(&mut self, block: &Block) -> Result<Value, RuntimeError> {
        self.push_scope();

        let mut last_value = Value::Null;

        for stmt in &block.statements {
            last_value = self.eval_statement(stmt)?;

            // Check for control flow
            if self.control_flow != ControlFlow::None {
                break;
            }
        }

        self.pop_scope();
        Ok(last_value)
    }

    /// Evaluate an expression
    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit, _) => Ok(self.eval_literal(lit)),
            Expr::Identifier(id) => self.get_variable(&id.name),
            Expr::Binary(binary) => self.eval_binary(binary),
            Expr::Unary(unary) => self.eval_unary(unary),
            Expr::Call(call) => self.eval_call(call),
            Expr::Index(index) => self.eval_index(index),
            Expr::ArrayLiteral(arr) => self.eval_array_literal(arr),
            Expr::Group(group) => self.eval_expr(&group.expr),
        }
    }

    /// Evaluate a literal
    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::string(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Null => Value::Null,
        }
    }

    /// Evaluate a binary expression
    fn eval_binary(&mut self, binary: &BinaryExpr) -> Result<Value, RuntimeError> {
        // Short-circuit evaluation for && and ||
        if binary.op == BinaryOp::And {
            let left = self.eval_expr(&binary.left)?;
            if let Value::Bool(false) = left {
                return Ok(Value::Bool(false));
            }
            if let Value::Bool(true) = left {
                let right = self.eval_expr(&binary.right)?;
                if let Value::Bool(b) = right {
                    return Ok(Value::Bool(b));
                }
            }
            return Err(RuntimeError::TypeError("Expected bool for &&".to_string()));
        }

        if binary.op == BinaryOp::Or {
            let left = self.eval_expr(&binary.left)?;
            if let Value::Bool(true) = left {
                return Ok(Value::Bool(true));
            }
            if let Value::Bool(false) = left {
                let right = self.eval_expr(&binary.right)?;
                if let Value::Bool(b) = right {
                    return Ok(Value::Bool(b));
                }
            }
            return Err(RuntimeError::TypeError("Expected bool for ||".to_string()));
        }

        // Regular binary operations
        let left = self.eval_expr(&binary.left)?;
        let right = self.eval_expr(&binary.right)?;

        match binary.op {
            BinaryOp::Add => match (&left, &right) {
                (Value::Number(a), Value::Number(b)) => {
                    let result = a + b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult);
                    }
                    Ok(Value::Number(result))
                }
                (Value::String(a), Value::String(b)) => {
                    Ok(Value::string(format!("{}{}", a, b)))
                }
                _ => Err(RuntimeError::TypeError("Invalid operands for +".to_string())),
            },
            BinaryOp::Sub => self.numeric_binary_op(left, right, |a, b| a - b),
            BinaryOp::Mul => self.numeric_binary_op(left, right, |a, b| a * b),
            BinaryOp::Div => {
                if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    let result = a / b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult);
                    }
                    Ok(Value::Number(result))
                } else {
                    Err(RuntimeError::TypeError("Expected numbers for /".to_string()))
                }
            }
            BinaryOp::Mod => {
                if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    let result = a % b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult);
                    }
                    Ok(Value::Number(result))
                } else {
                    Err(RuntimeError::TypeError("Expected numbers for %".to_string()))
                }
            }
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::Ne => Ok(Value::Bool(left != right)),
            BinaryOp::Lt => self.numeric_comparison(left, right, |a, b| a < b),
            BinaryOp::Le => self.numeric_comparison(left, right, |a, b| a <= b),
            BinaryOp::Gt => self.numeric_comparison(left, right, |a, b| a > b),
            BinaryOp::Ge => self.numeric_comparison(left, right, |a, b| a >= b),
            BinaryOp::And | BinaryOp::Or => {
                // Already handled above
                unreachable!()
            }
        }
    }

    /// Helper for numeric binary operations
    fn numeric_binary_op<F>(&self, left: Value, right: Value, op: F) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        if let (Value::Number(a), Value::Number(b)) = (left, right) {
            let result = op(a, b);
            if result.is_nan() || result.is_infinite() {
                return Err(RuntimeError::InvalidNumericResult);
            }
            Ok(Value::Number(result))
        } else {
            Err(RuntimeError::TypeError("Expected numbers".to_string()))
        }
    }

    /// Helper for numeric comparisons
    fn numeric_comparison<F>(
        &self,
        left: Value,
        right: Value,
        op: F,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        if let (Value::Number(a), Value::Number(b)) = (left, right) {
            Ok(Value::Bool(op(a, b)))
        } else {
            Err(RuntimeError::TypeError("Expected numbers for comparison".to_string()))
        }
    }

    /// Evaluate a unary expression
    fn eval_unary(&mut self, unary: &UnaryExpr) -> Result<Value, RuntimeError> {
        let operand = self.eval_expr(&unary.expr)?;

        match unary.op {
            UnaryOp::Negate => {
                if let Value::Number(n) = operand {
                    Ok(Value::Number(-n))
                } else {
                    Err(RuntimeError::TypeError("Expected number for -".to_string()))
                }
            }
            UnaryOp::Not => {
                if let Value::Bool(b) = operand {
                    Ok(Value::Bool(!b))
                } else {
                    Err(RuntimeError::TypeError("Expected bool for !".to_string()))
                }
            }
        }
    }

    /// Evaluate a function call
    fn eval_call(&mut self, call: &CallExpr) -> Result<Value, RuntimeError> {
        // Evaluate callee to get function name
        if let Expr::Identifier(id) = call.callee.as_ref() {
            let func_name = &id.name;

            // Evaluate arguments
            let args: Result<Vec<Value>, _> =
                call.args.iter().map(|arg| self.eval_expr(arg)).collect();
            let args = args?;

            // Check for stdlib functions first
            if crate::stdlib::is_builtin(func_name) {
                return crate::stdlib::call_builtin(func_name, &args)
                    .map_err(|_| RuntimeError::InvalidStdlibArgument);
            }

            // Check for user-defined functions
            if let Some(func) = self.functions.get(func_name).cloned() {
                return self.call_user_function(&func, args);
            }

            return Err(RuntimeError::UnknownFunction(func_name.clone()));
        }

        Err(RuntimeError::TypeError("Expected function name".to_string()))
    }

    /// Call a user-defined function
    fn call_user_function(
        &mut self,
        func: &UserFunction,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // Check arity
        if args.len() != func.params.len() {
            return Err(RuntimeError::TypeError(format!(
                "Function {} expects {} arguments, got {}",
                func.name,
                func.params.len(),
                args.len()
            )));
        }

        // Push new scope for function
        self.push_scope();

        // Bind parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            let scope = self.locals.last_mut().unwrap();
            scope.insert(param.name.name.clone(), arg.clone());
        }

        // Execute function body
        let mut result = Value::Null;
        for stmt in &func.body.statements {
            result = self.eval_statement(stmt)?;

            // Check for return
            if let ControlFlow::Return(val) = &self.control_flow {
                result = val.clone();
                self.control_flow = ControlFlow::None;
                break;
            }
        }

        self.pop_scope();
        Ok(result)
    }

    /// Evaluate array indexing
    fn eval_index(&mut self, index: &IndexExpr) -> Result<Value, RuntimeError> {
        let target = self.eval_expr(&index.target)?;
        let idx = self.eval_expr(&index.index)?;

        match target {
            Value::Array(arr) => {
                if let Value::Number(n) = idx {
                    let index_val = n as i64;
                    if n.fract() != 0.0 || n < 0.0 {
                        return Err(RuntimeError::InvalidIndex);
                    }

                    let borrowed = arr.borrow();
                    if index_val >= 0 && (index_val as usize) < borrowed.len() {
                        Ok(borrowed[index_val as usize].clone())
                    } else {
                        Err(RuntimeError::OutOfBounds)
                    }
                } else {
                    Err(RuntimeError::InvalidIndex)
                }
            }
            Value::String(s) => {
                if let Value::Number(n) = idx {
                    let index_val = n as i64;
                    if n.fract() != 0.0 || n < 0.0 {
                        return Err(RuntimeError::InvalidIndex);
                    }

                    let chars: Vec<char> = s.chars().collect();
                    if index_val >= 0 && (index_val as usize) < chars.len() {
                        Ok(Value::string(chars[index_val as usize].to_string()))
                    } else {
                        Err(RuntimeError::OutOfBounds)
                    }
                } else {
                    Err(RuntimeError::InvalidIndex)
                }
            }
            _ => Err(RuntimeError::TypeError("Cannot index non-array/string".to_string())),
        }
    }

    /// Evaluate array literal
    fn eval_array_literal(&mut self, arr: &crate::ast::ArrayLiteral) -> Result<Value, RuntimeError> {
        let elements: Result<Vec<Value>, _> =
            arr.elements.iter().map(|e| self.eval_expr(e)).collect();
        Ok(Value::array(elements?))
    }

    /// Get a variable value
    fn get_variable(&self, name: &str) -> Result<Value, RuntimeError> {
        // Check locals (innermost to outermost)
        for scope in self.locals.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Check globals
        self.globals
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::UndefinedVariable(name.to_string()))
    }

    /// Set a variable value
    fn set_variable(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        // Find in locals (innermost to outermost)
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }

        // Check globals
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return Ok(());
        }

        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }

    /// Set an array element
    fn set_array_element(
        &self,
        arr: Value,
        idx: Value,
        value: Value,
    ) -> Result<(), RuntimeError> {
        if let Value::Array(arr) = arr {
            if let Value::Number(n) = idx {
                let index_val = n as i64;
                if n.fract() != 0.0 || n < 0.0 {
                    return Err(RuntimeError::InvalidIndex);
                }

                let mut borrowed = arr.borrow_mut();
                if index_val >= 0 && (index_val as usize) < borrowed.len() {
                    borrowed[index_val as usize] = value;
                    Ok(())
                } else {
                    Err(RuntimeError::OutOfBounds)
                }
            } else {
                Err(RuntimeError::InvalidIndex)
            }
        } else {
            Err(RuntimeError::TypeError("Cannot index non-array".to_string()))
        }
    }

    /// Push a new scope
    fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    /// Pop the current scope
    fn pop_scope(&mut self) {
        self.locals.pop();
    }

    /// Define a global variable (for testing/REPL)
    pub fn define_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter_creation() {
        let mut interp = Interpreter::new();
        interp.define_global("x".to_string(), Value::Number(42.0));
        assert!(interp.globals.contains_key("x"));
    }

    #[test]
    fn test_eval_literal() {
        let interp = Interpreter::new();
        assert_eq!(interp.eval_literal(&Literal::Number(42.0)), Value::Number(42.0));
        assert_eq!(interp.eval_literal(&Literal::Bool(true)), Value::Bool(true));
        assert_eq!(interp.eval_literal(&Literal::Null), Value::Null);
    }

    #[test]
    fn test_scope_management() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.locals.len(), 1);

        interp.push_scope();
        assert_eq!(interp.locals.len(), 2);

        interp.pop_scope();
        assert_eq!(interp.locals.len(), 1);
    }
}
