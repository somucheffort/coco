use std::collections::BTreeMap;

use crate::{interpreter::{types::Value}};

use self::{io::IOModule, math::MathModule};

pub mod io;
pub mod math;

pub trait CocoModule {
    fn get() -> BTreeMap<String, Box<Value>>;
}

pub fn import_module(module: &str, objects: Option<Vec<String>>) -> Value {
    let lib = match module {
        "io" => IOModule::get(),
        "math" => MathModule::get(),
        _ => {
            // FIXME
            panic!("Unknown module: {}", module);
        }
    };

    if let Some(objects_some) = objects {
        return Value::Object(
            lib
            .into_iter()
            .filter(|val| objects_some.contains(&val.0))
            .collect()
        )
    }
    

    Value::Object(lib)

    
}