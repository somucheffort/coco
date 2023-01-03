use std::{collections::HashMap, env::Args, fmt};

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
                ("log".to_owned(), CocoValue::CocoFunction(vec![], Fun::Builtin(|vals| -> CocoValue {
                    for val in vals.iter() {
                        print!("{} ", val)
                    }
                    println!();
                    CocoValue::CocoNull
                })))
            ])
        }
    }

    pub fn get(&self, name: String) -> &CocoValue {
        self.variables.get(&name).unwrap()
    }

    pub fn set(&mut self, name: String, value: CocoValue) -> CocoValue {
        self.variables.insert(name, value).unwrap_or(CocoValue::CocoNull)
    }

    pub fn is_present(&self, name: String) -> bool {
        self.variables.contains_key(&name)
    }
}