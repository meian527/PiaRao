use crate::interpreter::{Function, FunctionImpl, Interpreter, ModuleFnPtr, Value};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

pub type ObjectRef = Arc<Object>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Object {
    Function { func: Function },
    String { data: String },
    Array { data: Vec<Value> },
    Record { id: usize, members: Box<[Value]> },
}
#[allow(dead_code)]
const OBJECT_FUNCTION_ID: usize = 0;
#[allow(dead_code)]
const OBJECT_STRING_ID: usize = 1;
#[allow(dead_code)]
const OBJECT_ARRAY_ID: usize = 2;
impl Object {
    #[allow(dead_code)]
    pub fn new_native_func(ptr: ModuleFnPtr) -> Self {
        Self::Function {
            func: Function {
                params: Vec::new(),
                body: FunctionImpl::Native(ptr),
            },
        }
    }
    #[allow(dead_code)]
    pub fn new_string_value(str: String) -> Value {
        Value::Object(ObjectRef::new(Self::String { data: str }))
    }
    #[allow(dead_code)]
    pub fn new_array_value(arr: Vec<Value>) -> Value {
        Value::Object(ObjectRef::new(Self::Array { data: arr }))
    }
    pub fn get_object_id(&self) -> usize {
        match self {
            Object::Function { func: _ } => 0usize,
            Object::String { data: _ } => 1usize,
            Object::Array { data: _ } => 2usize,
            Object::Record { id, members: _ } => *id,
        }
    }

    #[allow(dead_code)]
    pub fn is_same_type(&self, other: &Object) -> bool {
        self.get_object_id() == other.get_object_id()
    }
    #[allow(dead_code)]
    pub fn virtual_get_func<'a>(
        &self,
        name: &str,
        interp: &'a Interpreter,
    ) -> Option<&'a ObjectRef> {
        let id = self.get_object_id();
        interp.get_record_metadata()[id].member_funcs.get(name)
    }

    pub fn record_get_member_idx(id: usize, name: &str, interp: &Interpreter) -> Option<usize> {
        if let Some(members) = &interp.get_record_metadata()[id].members {
            members.get(name).cloned()
        } else {
            None
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String { data } => write!(f, "{}", data),
            _ => write!(f, "Object<address:{:p}>", self as *const _),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ObjectMetadata {
    pub name: String,
    pub size: usize,
    pub member_funcs: HashMap<String, ObjectRef>,
    pub members: Option<HashMap<String, usize>>,
}

#[allow(dead_code)]
pub fn new_record(id: usize, interp: &Interpreter) -> Option<ObjectRef> {
    let record = interp.get_record_metadata();
    if record.len() < id {
        let meta = &record[id];
        Some(ObjectRef::new(Object::Record {
            id,
            members: vec![Value::Null; meta.size].into_boxed_slice(),
        }))
    } else {
        None
    }
}
