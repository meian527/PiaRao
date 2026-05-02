use crate::ast;
use dashu_base::{Abs, Signed};
use dashu_ratio::RBig;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Value {
    Number(RBig),
    String(String),
    Lambda(Function),
    Null,
    Bool(bool),
}
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
}

#[allow(dead_code)]
pub enum ValueLiteral {
    Digit,
    Fraction,
    String,
    Bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    params: Vec<String>,
    body: ast::Expr,
}
impl Function {
    #[allow(dead_code)]
    pub fn new(params: Vec<String>, body: ast::Expr) -> Self {
        Self { params, body }
    }
    pub fn reset(&mut self, params: Vec<String>, body: ast::Expr) {
        self.params = params;
        self.body = body;
    }
}
static GLOBAL_MAIN_FUNC: Function = Function {
    params: Vec::new(),
    body: ast::Expr::Block(Vec::new()),
};

#[allow(dead_code)]
pub struct FunctionFrame {
    name: String,
    vars: HashMap<String, Value>,
    func: Option<Function>,
    last_ret_idx: usize,
}

pub struct Interpreter {
    // funcs: HashMap<String, RefCell<&'a Function>>,
    stack: Vec<Value>,
    frames: Vec<FunctionFrame>,
    pub prog: ast::Program,
    pc: usize,
    cur_func: Function,
    counter: usize,
}
impl Interpreter {
    pub fn new(prog: ast::Program) -> Self {
        Self {
            // funcs: HashMap::new(),
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
                        body: r.as_ref().clone(),
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
                let save_vars = self.frames.last().unwrap().vars.clone();
                for stmt in stmts.iter() {
                    self.eval_stmt(&stmt.stmt, stmt.l, stmt.c);
                }
                self.frames.last_mut().unwrap().vars = save_vars;
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
                        body: body.as_ref().clone(),
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
            _ => unimplemented!(),
        }
    }
    
    fn func_call(&mut self, name: &String, func: Function, args: &Vec<ast::Expr>, l: usize, c: usize) {
        self.cur_func.reset(func.params, func.body);
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
        let func_body = self.cur_func.body.clone();
        let last_pc = self.pc;
        self.pc = 0;
        // println!("calling: {}", name);
        self.eval_expr(&func_body, l, c);
        self.pc = last_pc;
        self.frames.pop();
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
    pub fn print_var(&self, name: &str, form: ValueLiteral) {
        for frame in self.frames.iter().rev() {
            if let Some(val) = frame.vars.get(name) {
                match form {
                    ValueLiteral::Digit => {
                        if let Value::Number(n) = val {
                            println!("{} = {}", name, Self::rbig_to_float_str(n, 10));
                            return;
                        }
                    }
                    ValueLiteral::String => {
                        if let Value::String(s) = val {
                            println!("{} = \"{}\"", name, s);
                            return;
                        }
                    }
                    ValueLiteral::Bool => {
                        if let Value::Bool(b) = val {
                            println!("{} = {}", name, b);
                            return;
                        }
                    }
                    ValueLiteral::Fraction => {
                        if let Value::Number(n) = val {
                            println!("{} = {}", name, n);
                            return;
                        }
                    }
                }
            }
        }
        println!("`{}` not found!", name);
    }
}
