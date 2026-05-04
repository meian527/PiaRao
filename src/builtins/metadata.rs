use crate::objects::{Object, ObjectMetadata};
use std::collections::HashMap;

pub fn get_builtin_metadata() -> Vec<ObjectMetadata> {
    vec![
        ObjectMetadata {
            name: "Function".to_string(),
            size: size_of::<Object>(),
            member_funcs: HashMap::new(),
            members: None,
        },
        ObjectMetadata {
            name: "String".to_string(),
            size: size_of::<Object>(),
            member_funcs: HashMap::new(),
            members: None,
        },
    ]
}
