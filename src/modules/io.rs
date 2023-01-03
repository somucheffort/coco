use std::{collections::BTreeMap, io::{ self, Write }};

use crate::interpreter::{scope::Scope, types::{CocoValue, Fun, FieldAccessor}};

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
            ("read".to_string(), Box::new(get_read())),
            ("stdin".to_string(), Box::new(get_stdin())),
            ("stdout".to_string(), Box::new(get_stdout()))
        ])
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
    CocoValue::CocoFunction(vec![], Fun::Builtin(|args| {
        for val in args.iter() {
            print!("{} ", val)
        }
        let _ = io::stdout().flush();
        let mut buffer = String::new();
        if let Ok(_b) = io::stdin().read_line(&mut buffer) {   
            return CocoValue::CocoString(buffer.trim_end().to_string())
        }
        CocoValue::CocoNull
    }))
}

fn get_stdout() -> CocoValue {
    CocoValue::CocoObject(
        BTreeMap::from([ 
            ("write".to_string(), Box::new(get_write())) 
        ])
    )
}

pub fn get_write() -> CocoValue {
    CocoValue::CocoFunction(vec![], Fun::Builtin(|args| {
        for val in args.iter() {
            print!("{} ", val)
        }
        println!();
        CocoValue::CocoNull
    }))
}