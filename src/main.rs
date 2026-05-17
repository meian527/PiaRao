mod ast;
mod builtins;
mod interpreter;
mod lang;
mod lexer;
mod objects;
mod parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return;
    }
    let src = std::fs::read_to_string(&args[1]).expect("should code file path");
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
    // let src = String::from(
    //     r#"
    //     let arr = ["1str1", "1str2", "1str3"];
    //     let str = arr[2].cat(arr[0]);
    //     println str;

    //     type Person = record name age;
    //     let person = Person "jack" 16;
    //     println person.name;
    //     "#,
    // );
    let mut lang_state = lang::LangState::new(&src);
    lang_state.register_builtins();
    // lang_state.print_ast();
    lang_state.interpret();
}
