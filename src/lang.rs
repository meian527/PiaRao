use crate::interpreter::Interpreter;
use crate::interpreter::ValueLiteral;
use crate::lexer;
use crate::parser;

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
    pub fn print_var(&self, name: &str, form: ValueLiteral) {
        self.interp.print_var(name, form);
    }
    #[allow(dead_code)]
    pub fn print_stack_top(&self) {
        self.interp.print_stack_top()
    }

    #[allow(dead_code)]
    pub fn print_ast(&self) {
        println!("{:#?}", self.interp.prog);
    }
}
