use std::{collections::HashMap};

use crate::modules::io;

use super::types::{CocoValue, FuncImpl, FuncArgs, FuncArg};

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
                ("num".to_owned(), CocoValue::CocoFunction(
                    "num".to_owned(),
                    FuncArgs::new(Vec::from([FuncArg::Required("any".to_string())])), 
                    FuncImpl::Builtin(|vals| {
                        CocoValue::CocoNumber(vals.get("any").unwrap().as_number())
                    })
                )),
                ("bool".to_owned(), CocoValue::CocoFunction(
                    "bool".to_owned(),
                    FuncArgs::new(Vec::from([FuncArg::Required("any".to_string())])), 
                    FuncImpl::Builtin(|vals| {
                        CocoValue::CocoBoolean(vals.get("any").unwrap().as_bool())
                    })
                )),
                ("str".to_owned(), CocoValue::CocoFunction(
                    "str".to_owned(),
                    FuncArgs::new(Vec::from([FuncArg::Required("any".to_string())])), 
                    FuncImpl::Builtin(|vals| {
                        CocoValue::CocoString(vals.get("any").unwrap().as_string())
                    })
                )),
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