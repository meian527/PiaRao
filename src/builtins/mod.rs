pub mod metadata;

use crate::interpreter::{ModuleFnPtr, ModuleFuncArgs, Value};
use crate::objects::{Object, ObjectRef};
use phf::phf_map;
use std::io::Write;

#[allow(dead_code, unpredictable_function_pointer_comparisons)]
pub(crate) static BUILTIN_FUNCTIONS: phf::Map<&str, ModuleFnPtr> = phf_map! {
    "print" => print,
    "println" => println,
    "input" => input,
    "type_info" => type_info,
    "to_string" => to_string,
    "__pie_rao_main__" => __pie_rao_main__,
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
