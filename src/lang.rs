use crate::interpreter::{Interpreter};
use crate::objects::{Object};
use crate::parser;
use crate::{builtins, lexer};

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
        return;
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
        for (name, ptr) in builtins::BUILTIN_FUNCTIONS.iter() {
            self.interp.new_func(name.to_string(), ptr.clone());
        }
        for i in 0..builtins::BUILTIN_RECORDS_FUNCTIONS.len() {
            let record_metadata_funcs = &mut self.interp.get_record_metadata_mut()[i].member_funcs;
            for (name, ptr) in builtins::BUILTIN_RECORDS_FUNCTIONS[i].iter() {
                record_metadata_funcs.insert(name.to_string(), Object::new_native_func(ptr.clone()));
            }
        }
    }
}
