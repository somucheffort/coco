use std::{collections::BTreeMap, io::{ self, Write }};

use crate::interpreter::{scope::Scope, types::{CocoValue, Fun, FieldAccessor}};

use super::CocoModule;

pub struct IOModule {}

impl CocoModule for IOModule {
    fn init(scope: &mut Scope, objects: Option<Vec<String>>) {
        let io = get_io();
        if objects.is_some() {
            for obj in objects.unwrap().iter() {
                let field_accessor = FieldAccessor::new(io.clone(), [CocoValue::CocoString(obj.to_string())].to_vec());
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
            ("stdin".to_string(), Box::new(get_stdin())) 
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