use crate::interpreter::{Function, ModuleFnPtr, Value};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ObjectRef = Arc<Object>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Object {
    Function { func: Function },
    String { data: String },
    Class { id: usize, members: Box<[Value]> },
}

#[allow(dead_code)]
pub struct ObjectMetadata {
    pub name: String,
    size: usize,
    member_funcs: HashMap<String, ModuleFnPtr>,
    members: Option<HashMap<String, usize>>,
}

lazy_static! {
    pub static ref GLOBAL_OBJECT_METADATA_MAP: Mutex<Vec<ObjectMetadata>> = Mutex::new(vec![
        ObjectMetadata {
            name: "Function".to_string(),
            size: size_of::<Object>(),
            member_funcs: HashMap::new(),
            members: None
        },
        ObjectMetadata {
            name: "String".to_string(),
            size: size_of::<Object>(),
            member_funcs: HashMap::new(),
            members: None
        },
    ]);
}

#[allow(dead_code)]
pub fn new_class_decl(
    name: String,
    member_funcs: HashMap<String, ModuleFnPtr>,
    members: Option<HashMap<String, usize>>,
) {
    GLOBAL_OBJECT_METADATA_MAP
        .lock()
        .unwrap()
        .push(ObjectMetadata {
            name,
            size: if let Some(members) = &members {
                members.len()
            } else {
                0
            },
            member_funcs,
            members,
        })
}

#[allow(dead_code)]
pub fn new_class(id: usize) -> Option<ObjectRef> {
    let class = &GLOBAL_OBJECT_METADATA_MAP.lock().unwrap();
    if class.len() < id {
        let meta = &class[id];
        Some(ObjectRef::new(Object::Class {
            id,
            members: vec![Value::Null; meta.size].into_boxed_slice(),
        }))
    } else {
        None
    }
}
