use crate::ast;
use dashu_base::{Abs, Signed};
use dashu_ratio::RBig;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
#[repr(C)]
pub enum Value {
    Number(RBig),
    String(String),
    Lambda(Function),
    Null,
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", Interpreter::rbig_to_float_str(n, 10)),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Lambda(func) => write!(f, "Lambda<address:{:p}>", func as *const _),
            Value::Null => write!(f, "Null"),
        }
    }
}

#[allow(dead_code)]
const VALUE_NUMBER: usize = 0;
#[allow(dead_code)]
const VALUE_STRING: usize = 1;
#[allow(dead_code)]
const VALUE_LAMBDA: usize = 2;
#[allow(dead_code)]
const VALUE_NULL: usize = 3;
#[allow(dead_code)]
const VALUE_BOOL: usize = 4;
impl Value {
    #[allow(dead_code)]
    pub fn set(&mut self, new_v: Value) -> Option<&Value> {
        match (self, new_v) {
            (Value::Number(x), Value::Number(y)) => *x = y.clone(),
            (Value::Lambda(x), Value::Lambda(y)) => *x = y.clone(),
            (Value::Bool(x), Value::Bool(y)) => *x = y.clone(),
            (Value::String(x), Value::String(y)) => *x = y.clone(),
            _ => return None,
        }
        None
    }
    pub fn type_info(&self) -> String {
        match self {
            Value::Number(_) => "Number".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Lambda(_) => "Lambda".to_string(),
            Value::Bool(_) => "Bool".to_string(),
            _ => "Null".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn string_to_type(s: &str) -> usize {
        match s {
            "Number" => VALUE_NUMBER,
            "String" => VALUE_STRING,
            "Lambda" => VALUE_LAMBDA,
            "Bool" => VALUE_BOOL,
            "Null" => VALUE_NULL,
            &_ => usize::MAX,
        }
    }
    pub fn type_to_string(id: usize) -> String {
        match id {
            VALUE_NUMBER => "Number".to_string(),
            VALUE_STRING => "String".to_string(),
            VALUE_LAMBDA => "Lambda".to_string(),
            VALUE_NULL => "Null".to_string(),
            VALUE_BOOL => "Bool".to_string(),
            _ => "<Unknown>".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn type_id(&self) -> usize {
        match self {
            Value::Number(_) => VALUE_NUMBER,
            Value::String(_) => VALUE_STRING,
            Value::Lambda(_) => VALUE_LAMBDA,
            Value::Bool(_) => VALUE_BOOL,
            Value::Null => VALUE_NULL,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
#[allow(dead_code)]
pub struct ModuleFuncArgs {
    pub args: Vec<Value>
}
impl ModuleFuncArgs {

    #[allow(dead_code)]
    pub fn new(args: Vec<Value>) -> Self {
        ModuleFuncArgs { args }
    }

    #[allow(dead_code)]
    pub fn check_types(&self, args: &[usize]) -> bool {
        for (marg, arg) in args.iter().zip(self.args.iter()) {
            if *marg != arg.type_id() {
                return false;
            }
        }
        true
    }
}

type ModuleFnPtr = unsafe fn(ModuleFuncArgs) -> Value;
#[allow(dead_code, unpredictable_function_pointer_comparisons)]
#[derive(Clone, Debug, PartialEq)]
pub enum FunctionImpl {
    General(ast::Expr),
    Native(Vec<usize>, ModuleFnPtr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    params: Vec<String>,
    body: FunctionImpl,
}
impl Function {
    #[allow(dead_code)]
    pub fn new(params: Vec<String>, body: FunctionImpl) -> Self {
        Self { params, body }
    }
    pub fn reset(&mut self, params: Vec<String>, body: FunctionImpl) {
        self.params = params;
        self.body = body;
    }
}
static GLOBAL_MAIN_FUNC: Function = Function {
    params: Vec::new(),
    body: FunctionImpl::General(ast::Expr::Block(Vec::new()))
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct FunctionFrame {
    name: String,
    vars: HashMap<String, Value>,
    func: Option<Function>,
    last_ret_idx: usize,
}

impl FunctionFrame {
    #[allow(dead_code)]
    pub fn reset(&mut self, name: &String, vars: &HashMap<String, Value>, func: Option<&Function>, last_ret_idx: usize) {
        self.name = name.clone();
        self.vars = vars.clone();
        self.func = func.cloned();
        self.last_ret_idx = last_ret_idx;
    }
    pub fn reset_to(&mut self, other: FunctionFrame) {
        self.name = other.name;
        self.vars = other.vars;
        self.func = other.func;
        self.last_ret_idx = other.last_ret_idx;
    }
}

#[allow(dead_code)]
pub struct Module {
    functions: Vec<Function>,
    sub_modules: Vec<Module>,
}

#[allow(dead_code)]
pub struct Interpreter {
    stack: Vec<Value>,
    frames: Vec<FunctionFrame>,
    loaded_modules: HashMap<String, Module>,
    pub prog: ast::Program,
    pc: usize,
    cur_func: Function,
    counter: usize,
}
impl Interpreter {
    pub fn new(prog: ast::Program) -> Self {
        Self {
            stack: Vec::new(),
            frames: {
                let mut result = Vec::new();
                result.push(FunctionFrame {
                    name: String::from("<main>"),
                    vars: HashMap::new(),
                    func: Some(GLOBAL_MAIN_FUNC.clone()),
                    last_ret_idx: usize::MAX,
                });
                result
            },
            prog,
            pc: 0,
            cur_func: GLOBAL_MAIN_FUNC.clone(),
            counter: 0,
            loaded_modules: HashMap::new(),
        }
    }
    pub fn interpret(&mut self) {
        let statements = self.prog.body.clone();
        for node in statements.iter() {
            self.eval_stmt(&node.stmt, node.l, node.c);
        }
    }
    fn error(&self, l: usize, c: usize, msg: &str) {
        println!("Traceback: ");
        let mut iter = self.frames.iter();
        iter.next();
        for frame in iter {
            println!("\tin function calling '{}()'", frame.name);
        }
        panic!("Error: at {}, {}\t: {}", l, c, msg);
    }
    fn eval_stmt(&mut self, node: &ast::Stmt, l: usize, c: usize) -> () {
        match node {
            ast::Stmt::TailReturn(expr) => {
                if self.frames.len() == 1 {
                    self.error(l, c, "Cannot return from <main> function");
                }
                self.eval_expr(expr.as_ref(), l, c);
                self.pc = self.frames.last().unwrap().last_ret_idx;
                self.frames.pop();
            }
            ast::Stmt::Expr(expr) => {
                self.eval_expr(expr.as_ref(), l, c);
                self.stack.pop();
            }
            ast::Stmt::Let(lhs, r) => {
                if let ast::Expr::Ident(name) = &**lhs {
                    // var
                    self.eval_expr(r, l, c);
                    let val = self.stack.pop().unwrap();
                    self.frames
                        .last_mut()
                        .unwrap()
                        .vars
                        .insert(name.clone(), val);
                } else if let ast::Expr::IdentList(params) = &**lhs {
                    // func
                    let name = &params[0];
                    let func = Function {
                        params: params[1..].to_vec(),
                        body: FunctionImpl::General(r.as_ref().clone()),
                    };
                    self.frames
                        .last_mut()
                        .unwrap()
                        .vars
                        .insert(name.clone(), Value::Lambda(func.clone()));
                } else {
                    self.error(
                        l,
                        c,
                        "Left-hand side of let statement must be an identifier",
                    );
                }
            }
        }
    }

    fn parse_float_string(s: &str) -> RBig {
        let mut parts = s.split('.');
        let int_part = parts.next().unwrap();
        let frac_part = parts.next().unwrap_or("");

        let denom = 10u64.pow(frac_part.len() as u32);

        let mut num = int_part.parse::<u64>().unwrap() * denom;
        if !frac_part.is_empty() {
            num += frac_part.parse::<u64>().unwrap();
        }

        RBig::from_parts(num.into(), denom.into())
    }
    fn eval_expr(&mut self, expr: &ast::Expr, l: usize, c: usize) -> () {
        match expr {
            ast::Expr::Number(n) => self.stack.push(Value::Number(RBig::from(*n))),
            ast::Expr::Float(n) => self.stack.push(Value::Number(Self::parse_float_string(n))),
            ast::Expr::Ident(s) => {
                if let Some(val) = self.find_var(s).cloned() {
                    self.stack.push(val);
                } else {
                    self.error(l, c, &format!("Undefined variable '{}'", s));
                }
            }
            ast::Expr::BinaryOp(lhs, op, rhs) => {
                self.eval_expr(lhs, l, c);
                match self.stack.pop().unwrap() {
                    Value::Number(r) => {
                        self.eval_expr(rhs, l, c);
                        if let Value::Number(ln) = self.stack.pop().unwrap() {
                            match op.as_str() {
                                "+" => self.stack.push(Value::Number(r + ln)),
                                "-" => self.stack.push(Value::Number(r - ln)),
                                "*" => self.stack.push(Value::Number(r * ln)),
                                "/" => self.stack.push(Value::Number(r / ln)),
                                "%" => self.stack.push(Value::Number(r / ln)),
                                "^" => self
                                    .stack
                                    .push(Value::Number(ln.pow(r.clone().try_into().unwrap()))),
                                "==" => self.stack.push(Value::Bool(r == ln)),
                                "!=" => self.stack.push(Value::Bool(r != ln)),
                                ">" => self.stack.push(Value::Bool(r > ln)),
                                "<" => self.stack.push(Value::Bool(r < ln)),
                                ">=" => self.stack.push(Value::Bool(r >= ln)),
                                "<=" => self.stack.push(Value::Bool(r <= ln)),
                                "=" => {
                                    //if let ast::Expr::Ident(name) = &**lhs {

                                    //}
                                }
                                _ => {
                                    self.error(l, c, &format!("Unknown operator '{}'", op));
                                }
                            }
                        }
                    }
                    _ => self.error(
                        l,
                        c,
                        &format!("Type error: `expr {} expr` op expects number operands", op),
                    ),
                }
            }
            ast::Expr::Assign(name, init) => {
                self.eval_expr(init, l, c);
                self.frames
                    .last_mut()
                    .unwrap()
                    .vars
                    .insert(name.clone(), self.stack.pop().unwrap());
            }
            ast::Expr::Block(stmts) => {
                let save_frame = self.frames.last().cloned().unwrap();
                for stmt in stmts.iter() {
                    self.eval_stmt(&stmt.stmt, stmt.l, stmt.c);
                }
                if let Some(frame_last) = self.frames.last_mut() {
                    frame_last.reset_to(save_frame);
                };
            }
            ast::Expr::Call(name, args) => {
                if let Some(Value::Lambda(func)) = self.find_var(name).cloned() {
                    self.func_call(name, func, args, l, c);
                } else {
                    self.error(l, c, &format!("Undefined function '{}'", name));
                }
            }
            ast::Expr::UnaryOp(op, expr) => {
                self.eval_expr(expr, l, c);
                match self.stack.pop().unwrap() {
                    Value::Number(n) => match op.as_str() {
                        "-" => self.stack.push(Value::Number(-n)),
                        "!" => self.stack.push(Value::Number(if n.is_zero() {
                            RBig::from(1)
                        } else {
                            RBig::from(0)
                        })),
                        _ => self.error(l, c, &format!("Unknown unary operator '{}'", op)),
                    },
                    _ => self.error(
                        l,
                        c,
                        &format!("Type error: `{}expr` op expects number operand", op),
                    ),
                }
            }
            ast::Expr::Null => return,
            ast::Expr::If(cond, then, els) => {
                self.eval_expr(cond, l, c);
                let cond_result = self.stack.pop().unwrap();
                if let Value::Bool(b) = cond_result {
                    if b {
                        self.eval_expr(then, l, c);
                    } else {
                        if let Some(els) = els {
                            self.eval_expr(els, l, c);
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                } else {
                    self.error(
                        l,
                        c,
                        &format!(
                            "if expression condition result must be `Bool` type but got `{}` type",
                            cond_result.type_info()
                        ),
                    );
                }
            }
            ast::Expr::Lambda(params, body) => {
                self.stack.push(Value::Lambda(
                    Function {
                        params: params.clone(),
                        body: FunctionImpl::General(body.as_ref().clone()),
                    }
                ));
            }
            ast::Expr::DynCall(func, args) => {
                self.eval_expr(func, l, c);
                if let Some(f) = self.stack.pop() {
                    if let Value::Lambda(func) = f {
                        self.func_call(&format!("Lambda<{}>", self.counter), func, args, l, c);
                    } else {
                        self.error(l, c, "This expression was not returned Lambda, cannot be called");
                    }
                } else {
                    self.error(l,c, "This expression was not returned value");
                }
            }
            ast::Expr::String(s) => {
                self.stack.push(Value::String(s.clone()));
            }
            _ => unimplemented!(),
        }
    }
    #[inline]
    fn func_call(&mut self, name: &String, func: Function, args: &Vec<ast::Expr>, l: usize, c: usize) {
        match &func.body {
            FunctionImpl::General(body) => {
                self.cur_func.reset(func.params, func.body.clone());
                let mut vars = HashMap::new();
                let iter = self.cur_func.params.clone();
                for (param, arg) in iter.into_iter().zip(args.into_iter()) {
                    self.eval_expr(arg, l, c);
                    vars.insert(param, self.stack.pop().unwrap());
                }
                self.frames.push(FunctionFrame {
                    name: name.clone(),
                    vars,
                    func: Some(self.cur_func.clone()),
                    last_ret_idx: self.pc,
                });
                let last_pc = self.pc;
                self.pc = 0;

                // println!("calling: {}", name);
                self.eval_expr(body, l, c);

                self.pc = last_pc;
                self.frames.pop();

            },
            FunctionImpl::Native(ts, ptr) => {
                let mut calling_args = Vec::<Value>::new();
                if ts.len() > 1 && ts[0] != usize::MAX {    //只有一个且为最大，就是不定长参数
                    for i in (0..=ts.len()).rev() {
                        if let Some(v) = self.stack.pop() {
                            if v.type_id() == ts[i] {
                                calling_args.push(v);
                            } else {
                                self.error(l, c, &format!("Type error: expected `{}` but got `{}`", Value::type_to_string(ts[i]), v.type_info()));
                            }
                        }
                    }
                } else {
                    for arg in args.iter() {
                        self.eval_expr(arg, l, c);
                        calling_args.push(self.stack.pop().unwrap());
                    }
                }
                unsafe {
                    self.stack.push(ptr(ModuleFuncArgs::new(calling_args)));
                }
            }
        }

    }

    #[allow(dead_code)]
    pub fn print_stack_top(&self) {
        if let Some(top) = self.stack.last() {
            println!("{:?}", top);
        } else {
            println!("Stack is empty");
        }
    }

    #[allow(dead_code)]
    pub fn find_var(&mut self, name: &str) -> Option<&Value> {
        for v in self.frames.last().unwrap().vars.iter() {
            if v.0 == name {
                return Some(v.1);
            }
        } //被调用者的帧
        for v in self.frames[self.frames.len() - 2].vars.iter() {
            if v.0 == name {
                return Some(v.1);
            }
        } //调用者的帧
        for v in self.frames.first().unwrap().vars.iter() {
            if v.0 == name {
                return Some(v.1);
            }
        } //全局作用域
        None
    }

    #[allow(dead_code)]
    fn rbig_to_float_str(r: &RBig, max_frac: usize) -> String {
        let mut num = r.numerator().clone();
        let denom = r.denominator().clone();

        let sign = if num.is_negative() { "-" } else { "" };
        num = num.abs();

        let int_part = &num / &denom;
        let mut rem = num % &denom;

        let mut frac = String::new();
        for _ in 0..max_frac {
            if rem.is_zero() {
                break;
            }
            rem *= 10;
            let digit = &rem / &denom;
            rem = rem % &denom;
            frac.push_str(&digit.to_string());
        }

        // 处理整数结果（如 4）
        if frac.is_empty() {
            format!("{}{}", sign, int_part)
        } else {
            format!("{}{}.{}", sign, int_part, frac)
        }
    }
    pub fn print_var(&self, name: &str) {
        for frame in self.frames.iter().rev() {
            if let Some(val) = frame.vars.get(name) {
                println!("`{}` = {}", name, val);
                return;
            }
        }
        println!("`{}` not found!", name);
    }

    #[allow(dead_code)]
    pub fn new_func(&mut self, name: String, ts: Vec<usize>, func: ModuleFnPtr) {
        self.frames[0].vars.insert(name.clone(), Value::Lambda(Function {
            params: vec!["@".to_string()],
            body: FunctionImpl::Native(ts, func),
        }));
    }

    #[allow(dead_code)]
    pub fn new_var(&mut self, name: String, var: Value) {
        self.frames[0].vars.insert(name, var);
    }
}
