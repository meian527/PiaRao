use crate::interpreter::{ModuleFnPtr, ModuleFuncArgs, Value};
use phf::phf_map;
use std::io::Write;

#[allow(dead_code, unpredictable_function_pointer_comparisons)]
pub(crate) static BUILTIN_FUNCTIONS: phf::Map<&str, ModuleFnPtr> = phf_map! {
    "print" => print,
    "println" => println,
    "input" => input,
};

fn print(args: ModuleFuncArgs) -> Value {
    for arg in args.args.iter() {
        print!("{}", arg);
    }
    // std::io::stdout().flush().expect("<stdout flush failure>");
    Value::Null
}

fn println(args: ModuleFuncArgs) -> Value {
    print(args);
    println!();
    Value::Null
}

fn input(args: ModuleFuncArgs) -> Value {
    let _ = print(args);
    std::io::stdout().flush().expect("<stdout flush failure>");
    let mut input = String::new();
    if std::io::stdin()
        .read_line(&mut input)
        .expect("<read line failure>")
        == 0
    {
        panic!("<stdin flush failure>");
    }
    // let input = input.trim_end().to_string();
    Value::String(input)
}
