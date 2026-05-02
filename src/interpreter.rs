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
        }
    }
    pub fn interpret(&mut self) {
        let statements = self.prog.body.clone();
        for node in statements.iter() {
            self.eval_stmt(&node.stmt, node.l, node.c);
        }
    }
    fn error(&self, l: usize, c: usize, msg: &str) {
        println!("Traceback: \n");
        for frame in self.frames.iter() {
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
                if let Some(val) = self.frames.last().unwrap().vars.get(s) {
                    self.stack.push(val.clone());
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
                                "+" => self.stack.push(Value::Number(ln + r)),
                                "-" => self.stack.push(Value::Number(ln - r)),
                                "*" => self.stack.push(Value::Number(ln * r)),
                                "/" => self.stack.push(Value::Number(ln / r)),
                                "%" => self.stack.push(Value::Number(ln % r)),
                                "^" => self
                                    .stack
                                    .push(Value::Number(ln.pow(r.clone().try_into().unwrap()))),
                                "=="  => self.stack.push(Value::Bool(ln == r)),
                                "!="  => self.stack.push(Value::Bool(ln != r)),
                                ">"   => self.stack.push(Value::Bool(ln > r)),
                                "<"   => self.stack.push(Value::Bool(ln < r)),
                                ">="  => self.stack.push(Value::Bool(ln >= r)),
                                "<="  => self.stack.push(Value::Bool(ln <= r)),    
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
                //self.cur_func = &*self.funcs.get(name).unwrap().borrow();
                if let Some(Value::Lambda(func)) =
                    self.frames.last().unwrap().vars.get(name).cloned()
                {
                    self.cur_func.reset(func.params, func.body);
                } else {
                    self.error(l, c, &format!("Undefined function '{}'", name));
                }
                // if let Some(func) = self.cur_func {
                // 	if func.params.len() != args.len() {
                // 		self.error(l, c, &format!("Function '{}' expects {} arguments but got {} arguments", name, func.params.len(), args.len()));
                // 	}
                // } else {
                // 	self.error(l, c, &format!("Undefined function '{}'", name));
                // }
                let mut vars = HashMap::new();
                let iter = self.cur_func.params.clone();
                for (param, arg) in iter.iter().zip(args.iter()) {
                    self.eval_expr(arg, l, c);
                    vars.insert(param.clone(), self.stack.pop().unwrap());
                }
                self.frames.push(FunctionFrame {
                    name: name.clone(),
                    vars: vars,
                    func: Some(self.cur_func.clone()),
                    last_ret_idx: self.pc,
                });
                let func_body = self.cur_func.body.clone();
                let last_pc = self.pc;
                self.pc = 0;
                self.eval_expr(&func_body, l, c);
                self.pc = last_pc;
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
            _ => unimplemented!(),
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
                return;
            }
        }
        println!("{} not found", name);
    }
}
