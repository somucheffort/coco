use std::{collections::BTreeMap, io::{ self, Write }, env};

use crate::interpreter::{scope::Scope, types::{CocoValue, FuncImpl, FieldAccessor, FuncArgs, FuncArg}};

use super::CocoModule;

pub struct IOModule {}

impl CocoModule for IOModule {
    fn init(scope: &mut Scope, objects: Option<Vec<String>>) {
        let io = get_io();
        if let Some(objects_some) = objects {
            for obj in objects_some.iter() {
                let field_accessor = FieldAccessor::new(io.clone(), Vec::from([CocoValue::CocoString(obj.to_string())]));
                scope.set(obj.to_string(), field_accessor.get());
            }
            return
        }
        scope.set("io".to_string(), io);
    }
}

fn get_io() -> CocoValue {
    CocoValue::CocoObject(
        BTreeMap::from([ 
            ("argv".to_string(), Box::new(get_argv())),
            ("read".to_string(), Box::new(get_read())),
            ("stdin".to_string(), Box::new(get_stdin())),
            ("stdout".to_string(), Box::new(get_stdout()))
        ])
    )
}

fn get_argv() -> CocoValue {
    CocoValue::CocoArray(
        env::args()
        .collect::<Vec<String>>()
        .drain(2..)
        .map(|s| Box::new(CocoValue::CocoString(s)))
        .collect::<Vec<Box<CocoValue>>>()
    )
}

fn get_stdin() -> CocoValue {
    CocoValue::CocoObject(
        BTreeMap::from([ 
            ("read".to_string(), Box::new(get_read())) 
        ])
    )
}

fn get_read() -> CocoValue {
    CocoValue::CocoFunction(
        FuncArgs::new(Vec::from([FuncArg::Spread("vals".to_string())])), 
        FuncImpl::Builtin(|args| {
            if let CocoValue::CocoArray(vals) = args.get("vals").unwrap() {
                for val in vals {
                    match *val.to_owned() {
                        CocoValue::CocoString(s) => print!("{} ", s),
                        _ => print!("{} ", val)
                    }
                }
            }
            let _ = io::stdout().flush();
            let mut buffer = String::new();
            if let Ok(_b) = io::stdin().read_line(&mut buffer) {   
                return CocoValue::CocoString(buffer.trim_end().to_string())
            }
            CocoValue::CocoNull
        })
    )
}

fn get_stdout() -> CocoValue {
    CocoValue::CocoObject(
        BTreeMap::from([ 
            ("write".to_string(), Box::new(get_write())) 
        ])
    )
}

pub fn get_write() -> CocoValue {
    CocoValue::CocoFunction(
        FuncArgs::new(Vec::from([FuncArg::Spread("vals".to_string())])), 
        FuncImpl::Builtin(|args| {
            if let CocoValue::CocoArray(vals) = args.get("vals").unwrap() {
                for val in vals {
                    match *val.to_owned() {
                        CocoValue::CocoString(s) => print!("{} ", s),
                        _ => print!("{} ", val)
                    }
                }
                println!()
            }

            CocoValue::CocoNull
        }
    ))
}