use crate::interpreter::Interpreter;
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
            self.interp
                .new_func(name.to_string(), vec![usize::MAX], ptr.clone());
        }
    }
}
