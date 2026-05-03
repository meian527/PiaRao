use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::interpreter::{Function, ModuleFnPtr, Value};

pub type ObjectRef = Arc<Object>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Object {
    Function {
        func: Function
    },
    String {
        data: String
    },
    Class {
        id: usize,
        members: HashMap<String, Value>,
    }
}

#[allow(dead_code)]
pub struct ObjectMetadata {
    pub name: String,
    size: usize,
    member_funcs: HashMap<String, ModuleFnPtr>,
}

lazy_static! {
    pub static ref GLOBAL_OBJECT_METADATA_MAP: Mutex<Vec<ObjectMetadata>> = Mutex::new(vec![
        ObjectMetadata { name: "Function".to_string(), size: size_of::<Object>(),member_funcs: HashMap::new() },
        ObjectMetadata { name: "String".to_string(), size: size_of::<Object>(),  member_funcs: HashMap::new() },  
    ]);
}

pub fn new_class_decl(name: String, members: HashMap<String, ModuleFnPtr>) {
    GLOBAL_OBJECT_METADATA_MAP.lock().unwrap().push( ObjectMetadata {
        name, size: members.len(), member_funcs: members
    })
}
