use crate::objects::{self, Object};
use crate::interpreter::{ModuleFnPtr, ModuleFuncArgs, Value};
use std::collections::HashMap;
use std::sync::LazyLock;

pub(crate) static BUILTIN_FUNCTIONS: LazyLock<HashMap<&str, ModuleFnPtr>> = LazyLock::new(|| {
    let mut m: HashMap<&str, ModuleFnPtr> = HashMap::new();
    m.insert("sub", std::sync::Arc::new(sub as fn(ModuleFuncArgs) -> Value));
    m.insert("cat", std::sync::Arc::new(cat as fn(ModuleFuncArgs) -> Value));
    m
});

pub fn sub(args: ModuleFuncArgs) -> Value {
    let args = args.args;
    if args.len() != 3 {
        panic!("`String::sub` called with incorrect number of arguments");
    }
    if let Value::Object(obj) = &args[0]
        && let Object::String { data } = obj.as_ref()
    {
        if let Value::Number(start) = &args[1]
            && let Value::Number(len) = &args[2]
        {
            let start = rug::Integer::from(start.numer() / start.denom())
                .to_i64_wrapping()
                as usize;
            let len = rug::Integer::from(len.numer() / len.denom())
                .to_i64_wrapping()
                as usize;
            return Object::new_string_value(
                data[start..start + len].to_string(),
            );
        } else {
            panic!("Invalid argument type");
        }
    }
    unreachable!()
}

pub fn cat(args: ModuleFuncArgs) -> Value {
    let args = args.args;
    if args.len() != 2 {
        panic!("`String::push` called with incorrect number of arguments");
    }
    let mut result = if let Value::Object(other) = &args[0] {
        if let Object::String { data } = other.as_ref() {
            data.clone()
        } else {
            panic!("Invalid argument type");
        }
    } else {
        panic!("Invalid argument type");
    };
    if let Value::Object(obj) = &args[1] {
        if let Object::String { data } = obj.as_ref() {
            result.push_str(data.as_str());
            return objects::Object::new_string_value(result);
        }
    }
    unreachable!()
}