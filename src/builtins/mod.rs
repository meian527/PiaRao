pub mod metadata;
pub mod string;

use crate::interpreter::{ModuleFnPtr, ModuleFuncArgs, Value};
use crate::objects::{Object, ObjectRef};
use std::collections::HashMap;
use std::io::Write;
use std::sync::LazyLock;

pub(crate) static BUILTIN_FUNCTIONS: LazyLock<HashMap<&str, ModuleFnPtr>> = LazyLock::new(|| {
    let mut m: HashMap<&str, ModuleFnPtr> = HashMap::new();
    m.insert("print", std::sync::Arc::new(print as fn(ModuleFuncArgs) -> Value));
    m.insert("println", std::sync::Arc::new(println as fn(ModuleFuncArgs) -> Value));
    m.insert("input", std::sync::Arc::new(input as fn(ModuleFuncArgs) -> Value));
    m.insert("type_info", std::sync::Arc::new(type_info as fn(ModuleFuncArgs) -> Value));
    m.insert("to_string", std::sync::Arc::new(to_string as fn(ModuleFuncArgs) -> Value));
    m.insert("__pie_rao_main__", std::sync::Arc::new(__pie_rao_main__ as fn(ModuleFuncArgs) -> Value));
    m
});
static FUNCTION_BUILTINS: LazyLock<HashMap<&str, ModuleFnPtr>> = LazyLock::new(|| HashMap::new());
pub(crate) static BUILTIN_RECORDS_FUNCTIONS: [&LazyLock<HashMap<&str, ModuleFnPtr>>; 2] = [
    &FUNCTION_BUILTINS,
    &string::BUILTIN_FUNCTIONS
];

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
    Value::Object(ObjectRef::new(Object::String { data: input }))
}

fn type_info(args: ModuleFuncArgs) -> Value {
    if args.args.len() != 1 {
        panic!("<type info failure>, `typeinfo()` only should 1 argument");
    }
    Value::Object(ObjectRef::new(Object::String {
        data: args.args[0].type_info(),
    }))
}

fn to_string(args: ModuleFuncArgs) -> Value {
    if args.args.len() != 1 {
        panic!("<type info failure>, `to_string()` only should 1 argument");
    }
    Value::Object(ObjectRef::new(Object::String {
        data: args.args[0].type_info().to_string(),
    }))
}

pub fn __pie_rao_main__(_: ModuleFuncArgs) -> Value {
    println(ModuleFuncArgs::new(vec![
        (Value::Object(ObjectRef::new(Object::String {
            data: "<main>".to_string(),
        }))),
    ])); // println "<main>";
    Value::Null
}
