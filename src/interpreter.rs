use crate::ast;
use crate::builtins;
use crate::objects;
use crate::objects::{Object, ObjectMetadata, ObjectRef};
use rug;
use rug::ops::Pow;
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt::Display;
use std::string::ToString;

type NumberImpl = rug::Rational;
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug, Clone)]
pub enum Value {
    Number(NumberImpl),
    Object(ObjectRef),
    Null,
    Bool(bool),
}
const VALUE_NUMBER: usize = 0;
const VALUE_OBJECT: usize = 1;
const VALUE_NULL: usize = 2;
const VALUE_BOOL: usize = 3;
// impl Clone for Value {
//     fn clone(&self) -> Value {
//         match self {
//             Value::Object(obj) => Value::Object(obj.clone()),
//             Value::Number(num) => Value::Number(num.clone()),
//             Value::Bool(bool) => Value::Bool(*bool),
//             Value::Null => Value::Null,
//         }
//     }
//
//     fn clone_from(&mut self, source: &Self)
//     where
//         Self:
//     {
//         todo!()
//     }
// }

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", Interpreter::rbig_to_float_str(n, 10)),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "Null"),
            Value::Object(obj) => write!(f, "{}", obj),
        }
    }
}

impl Value {
    #[allow(dead_code)]
    pub fn set(&mut self, new_v: Value) -> Option<&Value> {
        match (self, new_v) {
            (Value::Number(x), Value::Number(y)) => *x = y.clone(),
            (Value::Bool(x), Value::Bool(y)) => *x = y.clone(),
            _ => return None,
        }
        None
    }
    pub fn type_info(&self) -> String {
        match self {
            Value::Number(_) => "Number".to_string(),
            Value::Bool(_) => "Bool".to_string(),
            Value::Null => "Null".to_string(),
            Value::Object(_) => "Object".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn type_to_string(id: usize) -> String {
        match id {
            VALUE_NUMBER => "Number".to_string(),
            VALUE_BOOL => "Null".to_string(),
            VALUE_NULL => "Bool".to_string(),
            _ => "Object".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn type_id(&self) -> usize {
        match self {
            Value::Number(_) => VALUE_NUMBER,
            Value::Bool(_) => VALUE_BOOL,
            Value::Null => VALUE_NULL,
            Value::Object(_) => VALUE_OBJECT,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
#[allow(dead_code)]
pub struct ModuleFuncArgs {
    pub args: Vec<Value>,
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

pub type ModuleFnPtr = std::sync::Arc<dyn Fn(ModuleFuncArgs) -> Value + Send + Sync>;
#[allow(dead_code, unpredictable_function_pointer_comparisons)]
#[derive(Clone)]
pub enum FunctionImpl {
    General(ast::Stmt),
    Native(ModuleFnPtr),
}
impl std::fmt::Debug for FunctionImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General(stmt) => f.debug_tuple("General").field(stmt).finish(),
            Self::Native(_) => f.debug_tuple("Native").finish(),
        }
    }
}
impl PartialEq for FunctionImpl {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::General(a), Self::General(b)) => a == b,
            (Self::Native(_), Self::Native(_)) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub(crate) params: Vec<String>,
    pub(crate) body: FunctionImpl,
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
static GLOBAL_MAIN_FUNC: std::sync::LazyLock<Function> = std::sync::LazyLock::new(|| Function {
    params: Vec::new(),
    body: FunctionImpl::Native(std::sync::Arc::new(builtins::__pie_rao_main__ as fn(ModuleFuncArgs) -> Value)),
});

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
    pub fn reset(
        &mut self,
        name: &String,
        vars: &HashMap<String, Value>,
        func: Option<&Function>,
        last_ret_idx: usize,
    ) {
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

    #[allow(dead_code)]
    pub fn reset_vars(&mut self, vars: HashMap<String, Value>) {
        self.vars = vars;
    }

    #[allow(dead_code)]
    pub fn reinit(&mut self) {
        self.last_ret_idx = 0;
        self.func = None;
        self.vars.clear();
        self.name.clear();
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
    block_break_points: Vec<(bool, usize, FunctionFrame)>, // break_point, save_frame
    loop_continue_points: Vec<usize>,
    sp: usize,
    record_metadata: Vec<ObjectMetadata>,
    member_func_calling: bool,
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
                    func: Some((*GLOBAL_MAIN_FUNC).clone()),
                    last_ret_idx: usize::MAX,
                });
                result
            },
            prog,
            pc: 0,
            cur_func: (*GLOBAL_MAIN_FUNC).clone(),
            counter: 0,
            loaded_modules: HashMap::new(),
            block_break_points: Vec::new(),
            loop_continue_points: Vec::new(),
            sp: 0,
            record_metadata: builtins::metadata::get_builtin_metadata(),
            member_func_calling: false,
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
                // break impl, not only tail return
                if self.block_break_points.is_empty() {
                    self.error(l, c, "Cannot break block, maybe not in block");
                }
                let (_, last_pc, frame) = self.block_break_points.pop().unwrap();
                self.eval_expr(expr.as_ref(), l, c);
                self.pc = last_pc;
                self.frames.last_mut().unwrap().reset_to(frame);
            }
            ast::Stmt::Return(expr) => {
                if self.frames.len() == 1 {
                    self.error(l, c, "Cannot return from <main> function");
                }
                self.eval_expr(expr.as_ref(), l, c);
                self.pc = self.cur_frame().last_ret_idx;
                self.cur_frame_mut().reinit();
                self.sp -= 1;
            }
            ast::Stmt::Expr(expr) => {
                self.eval_expr(expr.as_ref(), l, c);
                let _ = self.stack.pop();
            }
            ast::Stmt::Let(lhs, r) => {
                if let ast::Expr::Ident(name) = &**lhs
                    && let ast::Stmt::NotPopValueExpr(expr) = &r.stmt
                {
                    // var
                    self.eval_expr(expr.as_ref(), r.l, r.c);
                    let val = self.stack.pop().unwrap();
                    self.cur_frame_mut().vars.insert(name.clone(), val);
                } else if let ast::Expr::IdentList(params) = &**lhs {
                    // func
                    let name = &params[0];
                    let func = Function {
                        params: params[1..].to_vec(),
                        body: FunctionImpl::General(r.as_ref().stmt.clone()),
                    };
                    self.frames.last_mut().unwrap().vars.insert(
                        name.clone(),
                        Value::Object(ObjectRef::new(Object::Function { func: func.clone() })),
                    );
                } else {
                    self.error(
                        l,
                        c,
                        "Left-hand side of let statement must be an identifier",
                    );
                }
            }
            ast::Stmt::NotPopValueExpr(expr) => {
                self.eval_expr(expr.as_ref(), l, c);
            }
            ast::Stmt::ADTypeDecl(_) => {
                unimplemented!()
            }
            ast::Stmt::RecordTypeDecl(name, members) => {
                let members = {
                    let mut result = HashMap::new();
                    for i in 0..members.len() {
                        result.insert(members[i].clone(), i);
                    }
                    result
                };
                let size = members.len();
                let id = self.record_metadata.len();
                self.record_metadata.push(objects::ObjectMetadata {
                    name: name.clone(), size, member_funcs: HashMap::new(), 
                    members: if members.is_empty() {
                        None
                    } else {
                        Some(members)
                    }
                });
                
                self.new_func(name.clone(), std::sync::Arc::new(move |args| {
                    objects::Object::new_record_value(id, args.args)
                }));
            }
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn cur_frame(&self) -> &FunctionFrame {
        &self.frames[self.sp]
    }

    #[allow(dead_code)]
    #[inline]
    pub fn cur_frame_mut(&mut self) -> &mut FunctionFrame {
        &mut self.frames[self.sp]
    }

    fn parse_float_string(s: &str) -> NumberImpl {
        let mut parts = s.split('.');
        let int_part = parts.next().unwrap();
        let frac_part = parts.next().unwrap_or("");

        NumberImpl::from(int_part.parse::<i64>().unwrap())
            + NumberImpl::from((
                frac_part.parse::<i64>().unwrap(),
                10_i64.pow(frac_part.len() as u32),
            ))
    }
    fn eval_expr(&mut self, expr: &ast::Expr, l: usize, c: usize) -> () {
        match expr {
            ast::Expr::Number(n) => self.stack.push(Value::Number(NumberImpl::from(*n))),
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
                match &self.stack.pop().unwrap() {
                    Value::Number(r) => {
                        self.eval_expr(rhs, l, c);
                        if let Value::Number(ln) = &self.stack.pop().unwrap() {
                            match op.as_str() {
                                "+" => self.stack.push(Value::Number(NumberImpl::from(r + ln))),
                                "-" => self.stack.push(Value::Number(NumberImpl::from(r - ln))),
                                "*" => self.stack.push(Value::Number(NumberImpl::from(r * ln))),
                                "/" => self.stack.push(Value::Number(NumberImpl::from(r / ln))),
                                "%" => self.stack.push(Value::Number(NumberImpl::from(r / ln))),
                                "^" => {
                                    let exp = if ln <= &i32::MAX {
                                        rug::Integer::from(ln.numer() / ln.denom())
                                            .to_i32()
                                            .unwrap()
                                    } else {
                                        self.error(l, c, &format!("`base ^ exp` exp so big(Max is int32::max but exp = {})", ln));
                                        0
                                    };
                                    self.stack.push(Value::Number(NumberImpl::from(r.pow(exp))))
                                }
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
                if let Some(Value::Object(func)) = self.find_var(name).cloned() {
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
                        _ => self.error(l, c, &format!("Unknown unary operator '{}'", op)),
                    },
                    Value::Bool(b) => match op.as_str() {
                        "!" => self.stack.push(Value::Bool(!b)),
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
                self.stack
                    .push(Value::Object(ObjectRef::new(Object::Function {
                        func: Function {
                            params: params.clone(),
                            body: FunctionImpl::General(ast::Stmt::Expr(Box::new(
                                body.as_ref().clone(),
                            ))),
                        },
                    })));
            }
            ast::Expr::DynCall(func, args) => {
                self.eval_expr(func, l, c);
                if let Some(f) = self.stack.pop() {
                    if let Value::Object(func) = f {
                        self.func_call(&format!("Lambda<{}>", self.counter), func, args, l, c);
                    } else {
                        self.error(
                            l,
                            c,
                            "This expression was not returned Lambda, cannot be called",
                        );
                    }
                } else {
                    self.error(l, c, "This expression was not returned value");
                }
            }
            ast::Expr::String(s) => {
                self.stack.push(Object::new_string_value(s.clone()));
            }
            ast::Expr::Dot(left, right) => {
                self.eval_expr(left, l, c);
                let this = self.stack.pop().unwrap();
                if let Value::Object(obj) = &this {
                    match right.as_ref() {
                        ast::Expr::Ident(name) => {
                            if let Some(func) = obj.virtual_get_func(name, self).cloned() {
                                self.stack.push(this.clone());
                                self.stack.push(Value::Object(func.clone()));
                                self.member_func_calling = true;
                            } else if let Object::Record { id, members } = obj.as_ref() {
                                if let Some(idx) = Object::record_get_member_idx(*id, name, self) {
                                    self.stack.push(members[idx].clone());
                                } else {
                                    self.error(
                                        l,
                                        c,
                                        &format!("this object not have name is `{}` member", name),
                                    );
                                }
                            } else {
                                self.error(
                                    l,
                                    c,
                                    &format!(
                                        "this `{}` not is member_func and not is record_member",
                                        name
                                    ),
                                );
                            }
                        }
                        _ => unreachable!(),
                    }
                } else {
                    self.error(l, c, "this `.` left not return object");
                }
            }
            ast::Expr::IdentList(_) => {
                unreachable!();
            }
            ast::Expr::Array(exprs) => {
                let mut values = Vec::new();
                for expr in exprs {
                    self.eval_expr(expr, l, c);
                    values.push(self.stack.pop().unwrap());
                }
                self.stack.push(Object::new_array_value(values));
            }
            ast::Expr::ArrayAccess(expr, index) => {
                self.eval_expr(expr, l, c);
                if let Value::Object(obj) = self.stack.pop().unwrap()
                    && let Object::Array { data: arr_data } = obj.as_ref() {
                    self.eval_expr(index, l, c);
                    if let Value::Number(number) = &self.stack.pop().unwrap() {
                        let index = rug::Integer::from(number.numer() / number.denom()).to_i64_wrapping() as usize;
                        let value = match arr_data.get(index) {
                            Some(value) => value.clone(),
                            None => Value::Null,
                        };
                        self.stack.push(value);
                    } else {
                        self.error(l, c, "this index expr result is not Number type");
                    }
                } else {
                    self.error(l, c, "this not is Array type");
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn new_record_decl(
        &mut self,
        name: String,
        member_funcs: HashMap<String, ObjectRef>,
        members: Option<HashMap<String, usize>>,
    ) {
        self.record_metadata.push(ObjectMetadata {
            name,
            size: if let Some(members) = &members {
                members.len()
            } else {
                0
            },
            member_funcs,
            members,
        })
    }

    #[allow(dead_code)]
    pub fn get_record_metadata_mut(&mut self) -> &mut Vec<ObjectMetadata> {
        &mut self.record_metadata
    }

    #[allow(dead_code)]
    pub fn get_record_metadata(&self) -> &Vec<ObjectMetadata> {
        &self.record_metadata
    }

    #[inline]
    pub fn func_calling(
        &mut self,
        name: &String,
        func: Function,
        args: &Vec<ast::Expr>,
        l: usize,
        c: usize,
    ) {
        match &func.body {
            FunctionImpl::General(body) => {
                self.cur_func.reset(func.params, func.body.clone());
                let name = String::from(name);
                let func: Option<Function> = Some(self.cur_func.clone());
                let last_ret_idx: usize = self.pc;

                let iter = self.cur_func.params.clone();
                let not_have_reuse_frame = self.sp == self.frames.len() - 1;
                if not_have_reuse_frame {
                    // 没有可复用的帧
                    let mut vars = HashMap::new();
                    if self.member_func_calling {
                        vars.insert("self".to_string(), self.stack.pop().unwrap());
                    }
                    for (param, arg) in iter.into_iter().zip(args.into_iter()) {
                        self.eval_expr(arg, l, c);

                        vars.insert(param, self.stack.pop().unwrap());
                    }
                    self.frames.push(FunctionFrame {
                        name,
                        vars,
                        func,
                        last_ret_idx,
                    });
                    self.sp += 1;
                } else {
                    self.sp += 1;

                    for (param, arg) in iter.into_iter().zip(args.into_iter()) {
                        self.eval_expr(arg, l, c);
                        let result = self.stack.pop().unwrap();
                        self.cur_frame_mut().vars.insert(param, result);
                    }
                    let cur_frame = self.cur_frame_mut();
                    cur_frame.name = name;
                    cur_frame.func = func;
                }

                self.pc = 0;

                // println!("calling: {}", name);
                self.eval_stmt(body, l, c);

                self.pc = self.cur_frame_mut().last_ret_idx;
                if not_have_reuse_frame {
                    self.frames.pop();
                }
                self.sp -= 1;
            }
            FunctionImpl::Native(ptr) => {
                let mut calling_args = Vec::new();
                if self.member_func_calling {
                    calling_args.push(self.stack.pop().unwrap());
                }
                for arg in args.iter() {
                    self.eval_expr(arg, l, c);
                    calling_args.push(self.stack.pop().unwrap());
                }
                self.stack.push(ptr(ModuleFuncArgs::new(calling_args)));
            }
        }
        self.member_func_calling = false;
    }
    #[inline]
    fn func_call(
        &mut self,
        name: &String,
        func_ref: ObjectRef,
        args: &Vec<ast::Expr>,
        l: usize,
        c: usize,
    ) {
        if let Object::Function { func } = func_ref.as_ref() {
            self.func_calling(name, func.clone(), args, l, c);
        } else {
            self.error(l, c, &format!("Undefined function '{}'", name));
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
    #[inline]
    pub fn find_var(&mut self, name: &str) -> Option<&Value> {
        for v in self.cur_frame().vars.iter() {
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
    fn rbig_to_float_str(r: &NumberImpl, max_frac: usize) -> String {
        let mut num = r.numer().clone();
        let denom = r.denom().clone();

        let sign = if num.is_negative() { "-" } else { "" };
        num = num.abs();

        let int_part = NumberImpl::from(&num / &denom);
        let mut rem = num % &denom;

        let mut frac = String::new();
        for _ in 0..max_frac {
            if rem.is_zero() {
                break;
            }
            rem *= 10;
            let digit = NumberImpl::from(&rem / &denom);
            rem = rem % &denom;
            frac.push_str(digit.to_string().as_str());
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
    pub fn new_func(&mut self, name: String, func: ModuleFnPtr) {
        self.frames[0].vars.insert(
            name.clone(),
            Value::Object(ObjectRef::new(Object::Function {
                func: Function {
                    params: vec!["@".to_string()],
                    body: FunctionImpl::Native(func),
                },
            })),
        );
    }

    #[allow(dead_code)]
    pub fn new_var(&mut self, name: String, var: Value) {
        self.frames[0].vars.insert(name, var);
    }
}
