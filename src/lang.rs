use crate::interpreter::{Function, FunctionImpl, Interpreter, Value};
use crate::parser;
use crate::{builtins, lexer};
use crate::objects::{Object, ObjectRef};

pub struct LangState {
    pub interp: Interpreter,
}
impl LangState {
    pub fn new(src: &String) -> Self {
        let prog = {
            let mut lexer = lexer::Lexer::new(&src);
            let tokens = lexer.tokenize();
            // for token in &tokens {
            // 	println!("{:?}", token);
            // }
            let mut parser = parser::Parser::new(&tokens);
            parser.parse()
        };
        Self {
            interp: Interpreter::new(prog),
        }
    }
    pub fn interpret(&mut self) {
        self.interp.interpret();
    }

    #[allow(dead_code)]
    pub fn print_var(&self, name: &str) {
        self.interp.print_var(name);
    }
    #[allow(dead_code)]
    pub fn print_stack_top(&self) {
        self.interp.print_stack_top()
    }

    #[allow(dead_code)]
    pub fn print_ast(&self) {
        println!("{:#?}", self.interp.prog);
    }

    #[allow(dead_code)]
    pub fn get_interp_mut(&mut self) -> &mut Interpreter {
        &mut self.interp
    }

    #[allow(dead_code)]
    pub fn register_builtins(&mut self) {
        for (name, ptr) in &builtins::BUILTIN_FUNCTIONS {
            self.interp.new_func(name.to_string(), ptr.clone());
        }
        self.get_interp_mut().get_record_metadata_mut()[1] // 1 是字符串类型的元信息id
            .member_funcs
            .insert(
                "sub".to_string(),
                ObjectRef::from(Object::Function {
                    func: Function {
                        params: Vec::new(), // native函数实现不需要这个
                        body: FunctionImpl::Native(|args| {
                            let args = args.args;
                            if let Value::Object(obj) = &args[0] && let Object::String { data } = obj.as_ref() {
                                if let Value::Number(start) = &args[1] && let Value::Number(len) = &args[2] {
                                    let start = rug::Integer::from(start.numer() / start.denom()).to_i64_wrapping() as usize;
                                    let len = rug::Integer::from(len.numer() / len.denom()).to_i64_wrapping() as usize;
                                    return Value::Object(ObjectRef::new(Object::String { data: data[start..start + len].to_string() }));
                                } else {
                                    panic!("Invalid argument type");
                                }
                            }
                            unreachable!()
                        }),
                    },
                }),
            );
    }
}
