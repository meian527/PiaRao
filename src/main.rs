mod ast;
mod interpreter;
mod lang;
mod lexer;
mod parser;

use std::time::Instant;

fn main() {
    let src = String::from(
        r#"
        let fib n = if n <= 1 then n else fib(n - 1) + fib(n - 2);
        let result = fib 30;
        "#,
    );
    let mut lang_state = lang::LangState::new(&src);
    //lang_state.print_ast();
    let start = Instant::now();
    lang_state.interpret();
    let duration = start.elapsed();
    println!("Took {:?}", duration);
    lang_state.print_var("result", interpreter::ValueLiteral::Digit);
}
