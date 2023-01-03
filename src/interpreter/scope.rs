use std::{collections::HashMap, env::Args, fmt};

use crate::modules::io;

use super::types::{CocoValue, Fun};

#[derive(Clone, Debug)]
pub struct Scope {
    previous: Option<Box<Scope>>,
    variables: HashMap<String, CocoValue>  
}

impl Scope {
    pub fn new(previous: Option<Box<Scope>>) -> Self {
        Self {
            previous,
            variables: HashMap::from([
                ("log".to_owned(), io::get_write()),
                ("number".to_owned(), CocoValue::CocoFunction(Vec::from(["any".to_owned()]), Fun::Builtin(|vals| {
                    CocoValue::CocoNumber(vals[0].as_number())
                })))
            ])
        }
    }

    pub fn get(&self, name: String) -> &CocoValue {
        let scope = self.find_scope(name.clone());
        
        scope.variables.get(&name).unwrap_or(&CocoValue::CocoNull)
    }

    pub fn set(&mut self, name: String, value: CocoValue) -> CocoValue {
        self.variables.insert(name, value).unwrap_or(CocoValue::CocoNull)
    }

    pub fn is_present(&self, name: String) -> bool {
        self.variables.contains_key(&name)
    }

    pub fn find_scope(&self, name: String) -> &Scope {
        let mut scope = self;
        while scope.previous.is_some() {
            if scope.is_present(name.clone()) {
                return scope
            }
            scope = self.previous.as_ref().unwrap()
        }

        scope
    }
}