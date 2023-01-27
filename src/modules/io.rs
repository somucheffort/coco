use std::{collections::BTreeMap, io::{ self, Write }, env};

use crate::interpreter::{scope::{Scope}, types::{Value, FuncImpl, FieldAccessor, FunctionArguments, FunctionArgument}};

use super::CocoModule;

pub struct IOModule {}

impl CocoModule for IOModule {
    fn init(scope: &mut Scope, objects: Option<Vec<String>>) {
        let io = get_io();

        if let Some(objects_some) = objects {
            for obj in objects_some.iter() {
                let mut field_accessor = FieldAccessor::new(io.clone(), Vec::from([Value::String(obj.to_string())]));
                let value = field_accessor.get(scope);
                scope.set(obj.to_string(), value);
            }
            return
        }
        scope.set("io".to_string(), io);
    }
}

fn get_io() -> Value {
    Value::Object(
        BTreeMap::from([ 
            ("argv".to_string(), Box::new(get_argv())),
            ("read".to_string(), Box::new(get_read())),
            ("stdin".to_string(), Box::new(get_stdin())),
            ("stdout".to_string(), Box::new(get_stdout()))
        ])
    )
}

fn get_argv() -> Value {
    Value::Array(
        env::args()
        .collect::<Vec<String>>()
        .drain(2..)
        .map(|s| Box::new(Value::String(s)))
        .collect::<Vec<Box<Value>>>()
    )
}

fn get_stdin() -> Value {
    Value::Object(
        BTreeMap::from([ 
            ("read".to_string(), Box::new(get_read())) 
        ])
    )
}

fn get_read() -> Value {
    Value::Function(
        "read".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Spread("vals".to_string())])), 
        FuncImpl::Builtin(|args| {
            if let Value::Array(vals) = args.get("vals").unwrap() {
                for val in vals {
                    match *val.to_owned() {
                        Value::String(s) => print!("{} ", s),
                        _ => print!("{} ", val)
                    }
                }
            }
            let _ = io::stdout().flush();
            let mut buffer = String::new();
            if let Ok(_b) = io::stdin().read_line(&mut buffer) {   
                return Value::String(buffer.trim_end().to_string())
            }
            Value::Null
        })
    )
}

fn get_stdout() -> Value {
    Value::Object(
        BTreeMap::from([ 
            ("write".to_string(), Box::new(get_write())) 
        ])
    )
}

pub fn get_write() -> Value {
    Value::Function(
        "write".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Spread("vals".to_string())])), 
        FuncImpl::Builtin(|args| {
            if let Value::Array(vals) = args.get("vals").unwrap() {
                for val in vals {
                    match *val.to_owned() {
                        Value::String(s) => print!("{} ", s),
                        _ => print!("{} ", val)
                    }
                }
                println!()
            }

            Value::Null
        }
    ))
}