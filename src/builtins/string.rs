use std::rc::Rc;
use phf::phf_map;
use crate::objects::Object;
use crate::interpreter::{ModuleFnPtr, ModuleFuncArgs, Value};

#[allow(dead_code, unpredictable_function_pointer_comparisons)]
pub(crate) static BUILTIN_FUNCTIONS: phf::Map<&str, ModuleFnPtr> = phf_map! {
    "sub" => sub,
    "push" => push
};

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

pub fn push(args: ModuleFuncArgs) -> Value {
    let mut args = args.args;
    if args.len() != 2 {
        panic!("`String::push` called with incorrect number of arguments");
    }
    let data_to_push = if let Value::Object(other) = &args[1] {
        if let Object::String { data } = other.as_ref() {
            data.clone()
        } else {
            panic!("Invalid argument type");
        }
    } else {
        panic!("Invalid argument type");
    };
    if let Value::Object(obj) = &mut args[0] {
        if let Object::String { data } = Rc::make_mut(obj) {
            data.push_str(data_to_push.as_str());
            return Value::Null;
        }
    }
    unreachable!()
}