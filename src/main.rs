mod ast;
mod interpreter;
mod lang;
mod lexer;
mod parser;

use std::time::Instant;
use crate::interpreter::{Value};

fn main() {
    // let src = String::from(
    //     r#"
    //     let fib = fn n -> if n <= 1 then n else fib(n - 1) + fib(n - 2);
    //     let fib2 n = if n <= 1 then n else fib2(n - 1) + fib2(n - 2);
    //     let result = fib 30;
    //     let fib2_result = fib2 10;
    //
    //     let add_result = (fn a b -> a + b)(4, 5);
    //     "#,
    // );
    let src = String::from(
        r#"
        print(fn -> 0);
        print((fn -> 4 + 5)());
        "#
    );
    let mut lang_state = lang::LangState::new(&src);
    lang_state.get_interp_mut().new_func("print".to_string(), vec![usize::MAX],  | args | {
        for arg in args.args.iter() {
            println!("{}", arg);
        }
        Value::Null
    });
    // lang_state.print_ast();
    let start = Instant::now();

    lang_state.interpret();
    let duration = start.elapsed();
    println!("Took {:?}", duration);
    // lang_state.print_var("result");
    // lang_state.print_var("fib2_result");
    // lang_state.print_var("add_result");
}
