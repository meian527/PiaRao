mod ast;
mod builtins;
mod interpreter;
mod lang;
mod lexer;
mod objects;
mod parser;
use crate::interpreter::{Function, FunctionImpl, Value};
use crate::objects::ObjectRef;
use std::time::Instant;

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
    // let src = String::from(
    //     r#"
    //     let fib n = if n <= 1 then n else fib(n - 1) + fib(n - 2);
    //     let result = fib 30;
    //     println "Yes";
    //     "#,
    // );
    let src = String::from(
        r#"
        let str = "str";
        str.trim();
        "#,
    );
    let mut lang_state = lang::LangState::new(&src);
    lang_state.register_builtins();

    lang_state.get_interp_mut().get_record_metadata_mut()[1] // 1 是字符串类型的元信息id
        .member_funcs
        .insert(
            "trim".to_string(),
            ObjectRef::from(objects::Object::Function {
                func: Function {
                    params: vec![], // native函数实现不需要这个
                    body: FunctionImpl::Native(|args| {
                        println!("I am a member func in String , my name is trim");
                        Value::Null
                    }),
                },
            }),
        );
    // lang_state.print_ast();
    let start = Instant::now();

    lang_state.interpret();
    let duration = start.elapsed();
    println!("Took {:?}", duration);
    lang_state.print_var("result");
}
