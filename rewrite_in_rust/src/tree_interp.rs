use crate::expr::*;
use crate::parser::*;
use crate::state::*;
use crate::stmt::*;
use crate::tokens::*;
use fnv::FnvHashMap;

trait InstanceValue {
    fn get(&mut self, name: TokenData) -> Value;
    fn set(&mut self, name: TokenData, val: Value);
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value;
}

#[derive(Debug, Clone)]
pub struct ObjectValue {}

impl InstanceValue for ObjectValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        Value::Empty
    }
}

#[derive(Debug, Clone)]
pub struct NumberValue {
    n: f64,
}

impl InstanceValue for NumberValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        if let Some(Value::Number(nv)) = val {
            return match op {
                Operator::Add => Value::Number(NumberValue { n: self.n + nv.n }),
                Operator::Sub => Value::Number(NumberValue { n: self.n - nv.n }),
                Operator::Pow => Value::Number(NumberValue {
                    n: self.n.powf(nv.n),
                }),
                Operator::Mul => Value::Number(NumberValue { n: self.n * nv.n }),
                Operator::Mod => Value::Number(NumberValue {
                    n: unsafe {
                        self.n.round().to_int_unchecked::<i64>()
                            % self.n.round().to_int_unchecked::<i64>()
                    } as f64,
                }),
                Operator::Div => Value::Number(NumberValue { n: self.n / nv.n }),
                Operator::Lt => Value::Bool(BoolValue { b: self.n < nv.n }),
                Operator::Lte => Value::Bool(BoolValue { b: self.n <= nv.n }),
                Operator::Gt => Value::Bool(BoolValue { b: self.n > nv.n }),
                Operator::Gte => Value::Bool(BoolValue { b: self.n >= nv.n }),
                Operator::Eq => Value::Bool(BoolValue { b: self.n == nv.n }),
                Operator::Neq => Value::Bool(BoolValue { b: self.n != nv.n }),
                _ => unreachable!(),
            };
        }
        // op == Operator::Neg
        return Value::Number(NumberValue { n: -self.n });
    }
}

#[derive(Debug, Clone)]
pub struct FnValue {
    declaration: FnStmt,
    is_initializer: bool,
    closure: *mut Env,
}

impl FnValue {
    fn call(&mut self, exec: &mut Exec, arguments: Vec<Value>) -> Result<Value, InterpreterError> {
        if arguments.len() != self.declaration.params.len() {
            return Err(InterpreterError {
                message: "Invalid number of arguments".to_string(),
                line: self.declaration.name.line,
                pos: 0,
            });
        }

        let mut env = Env {
            enclosing: Some(self.closure),
            values: FnvHashMap::default(),
        };

        for i in 0..self.declaration.params.len() {
            env.define(self.declaration.params[i].lexeme, arguments[i].clone());
        }

        let result = exec.execute_block(&self.declaration.body, core::ptr::addr_of_mut!(env));

        return Ok(result);
    }
}

impl InstanceValue for FnValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        Value::Empty
    }
}

#[derive(Debug, Clone)]
pub struct StringValue {
    s: String,
}

impl InstanceValue for StringValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        Value::Empty
    }
}

#[derive(Debug, Clone)]
pub struct BoolValue {
    b: bool,
}

impl InstanceValue for BoolValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        Value::Empty
    }
}

#[derive(Debug, Clone)]
pub struct ClassValue {}

impl InstanceValue for ClassValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        Value::Empty
    }
}

#[derive(Debug, Clone)]
pub struct DictValue {}

impl InstanceValue for DictValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        Value::Empty
    }
}

#[derive(Debug, Clone)]
pub struct ListValue {}

impl InstanceValue for ListValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        Value::Empty
    }
}

#[derive(Debug, Clone)]
pub struct NativeValue {}

impl InstanceValue for NativeValue {
    fn get(&mut self, name: TokenData) -> Value {
        Value::Empty
    }
    fn set(&mut self, name: TokenData, val: Value) {}
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        Value::Empty
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Object(ObjectValue),
    Number(NumberValue),
    Fn(FnValue),
    String(StringValue),
    Bool(BoolValue),
    Class(ClassValue),
    Dict(DictValue),
    List(ListValue),
    Native(NativeValue),
    Nil,
    Continue,
    Break,
    Return,
    Empty,
}

impl Value {
    fn get_variant_instance(&mut self) -> &mut dyn InstanceValue {
        match self {
            Value::Object(v) => v,
            Value::Number(v) => v,
            Value::Fn(v) => v,
            Value::String(v) => v,
            Value::Bool(v) => v,
            Value::Class(v) => v,
            Value::Dict(v) => v,
            Value::List(v) => v,
            Value::Native(v) => v,
            _ => panic!("Not posible"),
        }
    }
}

impl InstanceValue for Value {
    fn get(&mut self, name: TokenData) -> Value {
        self.get_variant_instance().get(name)
    }
    fn set(&mut self, name: TokenData, val: Value) {
        self.get_variant_instance().set(name, val)
    }
    fn perform_operation(&mut self, op: Operator, val: Option<Value>) -> Value {
        self.get_variant_instance().perform_operation(op, val)
    }
}

trait StmtVisitor {
    fn visit_expr_stmt(&mut self, stmt: &ExprStmt) -> Value;
    fn visit_try_catch_stmt(&mut self, stmt: &TryCatchStmt) -> Value;
    fn visit_classic_for_stmt(&mut self, stmt: &ClassicForStmt) -> Value;
    fn visit_enhanced_for_stmt(&mut self, stmt: &EnhancedForStmt) -> Value;
    fn visit_let_stmt(&mut self, stmt: &LetStmt) -> Value;
    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Value;
    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Value;
    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Value;
    fn visit_break_stmt(&mut self, stmt: &BreakStmt) -> Value;
    fn visit_continue_stmt(&mut self, stmt: &ContinueStmt) -> Value;
    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Value;
    fn visit_fn_stmt(&mut self, stmt: &FnStmt) -> Value;
    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> Value;
}

trait ExprVisitor {
    fn visit_function_expr(&mut self, expr: &FnExpr) -> Value;
    fn visit_variable_expr(&mut self, expr: &VarExpr) -> Value;
    fn visit_list_expr(&mut self, expr: &ListExpr) -> Value;
    fn visit_dictionary_expr(&mut self, expr: &DictionaryExpr) -> Value;
    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Value;
    fn visit_access_expr(&mut self, expr: &AccessExpr) -> Value;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Value;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> Value;
    fn visit_get_expr(&mut self, expr: &GetExpr) -> Value;
    fn visit_set_expr(&mut self, expr: &SetExpr) -> Value;
    fn visit_super_expr(&mut self, expr: &SuperExpr) -> Value;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Value;
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Value;
    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Value;
    fn visit_this_expr(&mut self, expr: &ThisExpr) -> Value;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Value;
}

impl Expr {
    fn accept(&self, visitor: &mut dyn ExprVisitor) -> Value {
        match self {
            Expr::Fn(expr) => visitor.visit_function_expr(&expr),
            Expr::Var(expr) => visitor.visit_variable_expr(&expr),
            Expr::List(expr) => visitor.visit_list_expr(&expr),
            Expr::Dictionary(expr) => visitor.visit_dictionary_expr(&expr),
            Expr::Assign(expr) => visitor.visit_assign_expr(&expr),
            Expr::Access(expr) => visitor.visit_access_expr(&expr),
            Expr::Binary(expr) => visitor.visit_binary_expr(&expr),
            Expr::Call(expr) => visitor.visit_call_expr(&expr),
            Expr::Get(expr) => visitor.visit_get_expr(&expr),
            Expr::Set(expr) => visitor.visit_set_expr(&expr),
            Expr::Super(expr) => visitor.visit_super_expr(&expr),
            Expr::Grouping(expr) => visitor.visit_grouping_expr(&expr),
            Expr::Literal(expr) => visitor.visit_literal_expr(&expr),
            Expr::Logical(expr) => visitor.visit_logical_expr(&expr),
            Expr::This(expr) => visitor.visit_this_expr(&expr),
            Expr::Unary(expr) => visitor.visit_unary_expr(&expr),
            Expr::Empty => unreachable!(),
        }
    }
}

impl Stmt {
    fn accept(&self, visitor: &mut dyn StmtVisitor) -> Value {
        match self {
            Stmt::Fn(stmt) => visitor.visit_fn_stmt(&stmt),
            Stmt::Let(stmt) => visitor.visit_let_stmt(&stmt),
            Stmt::Block(stmt) => visitor.visit_block_stmt(&stmt),
            Stmt::Class(stmt) => visitor.visit_class_stmt(&stmt),
            Stmt::ClassicFor(stmt) => visitor.visit_classic_for_stmt(&stmt),
            Stmt::EnhancedFor(stmt) => visitor.visit_enhanced_for_stmt(&stmt),
            Stmt::While(stmt) => visitor.visit_while_stmt(&stmt),
            Stmt::If(stmt) => visitor.visit_if_stmt(&stmt),
            Stmt::Continue(stmt) => visitor.visit_continue_stmt(&stmt),
            Stmt::Return(stmt) => visitor.visit_return_stmt(&stmt),
            Stmt::Break(stmt) => visitor.visit_break_stmt(&stmt),
            Stmt::TryCatch(stmt) => visitor.visit_try_catch_stmt(&stmt),
            Stmt::Expr(stmt) => visitor.visit_expr_stmt(&stmt),
        }
    }
}

#[derive(Debug)]
pub struct Env {
    pub enclosing: Option<*mut Env>,
    pub values: FnvHashMap<&'static str, Value>,
}

impl Env {
    fn new() -> Self {
        Self {
            enclosing: None,
            values: FnvHashMap::default(),
        }
    }

    fn define(&mut self, name: &'static str, val: Value) {
        self.values.insert(name, val);
    }

    fn get(&self, name: &'static str) -> Value {
        if let Some(val) = self.values.get(name) {
            return val.clone();
        }
        if let Some(enclosed) = self.enclosing {
            unsafe {
                return (*enclosed).get(name);
            }
        }
        panic!("Undefined var");
    }

    fn assign(&mut self, name: &'static str, val: Value) {
        if self.values.contains_key(name) {
            self.values.insert(name, val);
            return;
        }
        if let Some(enclosed) = self.enclosing {
            unsafe {
                (*enclosed).assign(name, val);
            }
            return;
        }
        panic!("Undefined var");
    }
}

#[derive(Debug)]
pub struct Exec {
    env: *mut Env,

    call_stack: Vec<CallStack>,
}

impl StmtVisitor for Exec {
    fn visit_expr_stmt(&mut self, stmt: &ExprStmt) -> Value {
        stmt.expression.accept(self)
    }
    fn visit_try_catch_stmt(&mut self, stmt: &TryCatchStmt) -> Value {
        return Value::Empty;
    }
    fn visit_classic_for_stmt(&mut self, stmt: &ClassicForStmt) -> Value {
        if let Some(init) = &stmt.initializer {
            init.accept(self);
        }
        while truthy(stmt.condition.accept(self)) {
            // TODO: implement break/continue
            stmt.body.accept(self);
            stmt.increment.accept(self);
        }
        // TODO: add enter/leave loop logic
        return Value::Empty;
    }
    fn visit_enhanced_for_stmt(&mut self, stmt: &EnhancedForStmt) -> Value {
        return Value::Empty;
    }

    fn visit_let_stmt(&mut self, stmt: &LetStmt) -> Value {
        let val: Value;
        if let Some(initializer) = &stmt.initializer {
            val = initializer.accept(self);
        } else {
            val = Value::Nil;
        }
        let val_copy = val.clone();
        unsafe { (*self.env).define(stmt.name.lexeme, val) };
        return val_copy;
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Value {
        let mut env = Env {
            enclosing: Some(self.env),
            values: FnvHashMap::default(),
        };
        self.execute_block(&stmt.stmts, core::ptr::addr_of_mut!(env));
        return Value::Empty;
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Value {
        self.enter_loop();
        while truthy(stmt.condition.accept(self)) {
            let val = stmt.body.accept(self);
            // TODO: handle return, break and continue
        }
        self.leave_loop();
        return Value::Empty;
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Value {
        return Value::Empty;
    }
    fn visit_break_stmt(&mut self, stmt: &BreakStmt) -> Value {
        return Value::Empty;
    }
    fn visit_continue_stmt(&mut self, stmt: &ContinueStmt) -> Value {
        return Value::Empty;
    }
    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Value {
        return Value::Empty;
    }
    fn visit_fn_stmt(&mut self, stmt: &FnStmt) -> Value {
        unsafe {
            (*self.env).define(
                &stmt.name.lexeme,
                Value::Fn(FnValue {
                    declaration: stmt.clone(),
                    // TODO: implement closure
                    is_initializer: false,
                    closure: self.env,
                }),
            )
        };
        return Value::Empty;
    }
    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> Value {
        return Value::Empty;
    }
}

impl ExprVisitor for Exec {
    fn visit_function_expr(&mut self, expr: &FnExpr) -> Value {
        return Value::Empty;
    }
    fn visit_variable_expr(&mut self, expr: &VarExpr) -> Value {
        let name = expr.name.as_ref().unwrap().lexeme;
        return unsafe { (*self.env).get(name) };
    }
    fn visit_list_expr(&mut self, expr: &ListExpr) -> Value {
        return Value::Empty;
    }
    fn visit_dictionary_expr(&mut self, expr: &DictionaryExpr) -> Value {
        return Value::Empty;
    }

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Value {
        let val = expr.value.accept(self);
        // TODO: implement assignment for dict and list
        unsafe { (*self.env).assign(&expr.name.lexeme, val) };
        return Value::Empty;
    }

    fn visit_access_expr(&mut self, expr: &AccessExpr) -> Value {
        return Value::Empty;
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Value {
        let mut left = expr.left.accept(self);
        let right = expr.right.accept(self);
        let op = match expr.operator.token {
            Token::EqualEqual => Operator::Eq,
            Token::BangEqual => Operator::Neq,
            Token::Greater => Operator::Gt,
            Token::GreaterEqual => Operator::Gte,
            Token::Less => Operator::Lt,
            Token::LessEqual => Operator::Lte,
            Token::Plus => Operator::Add,
            Token::Minus => Operator::Sub,
            Token::Slash => Operator::Div,
            Token::Mod => Operator::Mod,
            Token::Star => Operator::Mul,
            Token::Power => Operator::Pow,
            _ => unreachable!(),
        };
        left.perform_operation(op, Some(right))
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> Value {
        return Value::Empty;
    }
    fn visit_get_expr(&mut self, expr: &GetExpr) -> Value {
        return Value::Empty;
    }
    fn visit_set_expr(&mut self, expr: &SetExpr) -> Value {
        return Value::Empty;
    }
    fn visit_super_expr(&mut self, expr: &SuperExpr) -> Value {
        return Value::Empty;
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Value {
        expr.expression.accept(self)
    }

    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Value {
        match &expr.value {
            Literal::Boolean(b) => Value::Bool(BoolValue { b: *b }),
            Literal::String(s) => Value::String(StringValue { s: s.clone() }),
            Literal::Number(n) => Value::Number(NumberValue { n: *n }),
            Literal::Nil => Value::Nil,
        }
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Value {
        let left = truthy(expr.left.accept(self));
        if expr.operator.token == Token::Or {
            if left {
                return Value::Bool(BoolValue { b: true });
            }
            let right = truthy(expr.right.accept(self));
            return Value::Bool(BoolValue { b: left || right });
        }
        // expr.operator.token = AND
        if !left {
            return Value::Bool(BoolValue { b: false });
        }
        let right = truthy(expr.right.accept(self));
        return Value::Bool(BoolValue { b: left && right });
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr) -> Value {
        return Value::Empty;
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Value {
        let mut value = expr.right.accept(self);
        match expr.operator.token {
            Token::Not => Value::Bool(BoolValue { b: truthy(value) }),
            Token::Minus => self.operate_unary(Operator::Neg, &mut value),
            _ => unreachable!(),
        }
    }
}

impl Exec {
    pub fn new(env: *mut Env) -> Exec {
        return Exec {
            call_stack: vec![],
            env: env,
        };
    }

    fn enter_function(&mut self, name: String) {
        self.call_stack.push(CallStack {
            function: name,
            loop_count: 0,
        });
    }

    fn leave_function(&mut self, name: String) {
        let pc = self.call_stack.last().unwrap();
        if pc.function != name {
            panic!("Leaving function doesn't match with top stack");
        }
        self.call_stack.pop();
    }

    fn enter_loop(&mut self) {
        let mut pc = self.call_stack.last_mut().unwrap();
        pc.loop_count += 1;
    }

    fn leave_loop(&mut self) {
        let mut pc = self.call_stack.last_mut().unwrap();
        pc.loop_count -= 1;
    }

    fn inside_loop(&mut self) -> bool {
        return self.call_stack.last().unwrap().loop_count != 0;
    }

    pub fn interpret(&mut self, stmts: &mut Vec<Stmt>) {
        self.enter_function("".to_string());
        for s in stmts {
            let _val = s.accept(self);
            // println!("Executing {:#?}", _val);
        }
    }

    fn execute_block(&mut self, stmts: &Vec<Stmt>, env: *mut Env) -> Value {
        let previous = self.env;
        self.env = env;
        for s in stmts {
            s.accept(self);
            // TODO: handle continue,break and return
        }
        self.env = previous;
        return Value::Empty;
    }

    fn operate_unary(&mut self, op: Operator, val: &mut Value) -> Value {
        val.perform_operation(op, None)
    }
}

fn truthy(val: Value) -> bool {
    match val {
        Value::String(v) => v.s.len() > 0,
        Value::Number(v) => v.n != 0.0,
        Value::Bool(v) => v.b,
        Value::Nil => false,
        _ => true,
    }
}
