use std::{collections::HashMap};

use super::types::CocoValue;

#[derive(Clone, Debug)]
pub struct Scope {
    previous: Option<Box<Scope>>,
    variables: HashMap<String, CocoValue>   
}

impl Scope {
    pub fn new(previous: Option<Box<Scope>>) -> Self {
        Self {
            previous,
            variables: HashMap::new()
        }
    }

    pub fn get(&self, name: String) -> &CocoValue {
        self.variables.get(&name).unwrap()
    }

    pub fn set(&mut self, name: String, value: CocoValue) -> CocoValue {
        self.variables.insert(name, value).unwrap_or(CocoValue::CocoNull)
    }
}