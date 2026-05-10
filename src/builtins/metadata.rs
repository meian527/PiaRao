use crate::objects::ObjectMetadata;
use std::collections::HashMap;

pub fn get_builtin_metadata() -> Vec<ObjectMetadata> {
    vec![
        ObjectMetadata {
            name: "Function".to_string(),
            size: 0,
            member_funcs: HashMap::new(),
            members: None,
        },
        ObjectMetadata {
            name: "String".to_string(),
            size: 0,
            member_funcs: HashMap::new(),
            members: None,
        },
        ObjectMetadata {
            name: "Array".to_string(),
            size: 0,
            member_funcs: HashMap::new(),
            members: None,
        },
    ]
}
