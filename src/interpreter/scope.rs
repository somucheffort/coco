use std::{collections::HashMap, process::exit};

use colored::Colorize;
use lazy_static::lazy_static;

use crate::modules::io;

use super::types::{Value, FuncImpl, FunctionArguments, FunctionArgument};

lazy_static! {
    static ref STD: HashMap<String, Value> = HashMap::from([
        ("log".to_owned(), io::get_write()),
        ("num".to_owned(), Value::Function(
            "num".to_owned(),
            FunctionArguments::new(Vec::from([FunctionArgument::Required("any".to_string())])), 
            FuncImpl::Builtin(|vals| {
                Value::Number(vals.get("any").unwrap().as_number())
            })
        )),
        ("bool".to_owned(), Value::Function(
            "bool".to_owned(),
            FunctionArguments::new(Vec::from([FunctionArgument::Required("any".to_string())])), 
            FuncImpl::Builtin(|vals| {
                Value::Boolean(vals.get("any").unwrap().as_bool())
            })
        )),
        ("str".to_owned(), Value::Function(
            "str".to_owned(),
            FunctionArguments::new(Vec::from([FunctionArgument::Required("any".to_string())])), 
            FuncImpl::Builtin(|vals| {
                Value::String(vals.get("any").unwrap().as_string())
            })
        )),
    ]);
}

#[derive(Clone, Debug)]
pub struct Scope {
    previous: Option<Box<Scope>>,
    variables: HashMap<String, Value>,
    pub filename: String
}

impl Scope {
    pub fn new(filename: String) -> Self {
        Self::from(None, filename)
    }

    pub fn from(previous: Option<Box<Scope>>, filename: String) -> Self {
        Self {
            previous,
            variables: HashMap::from([
                ("log".to_owned(), io::get_write()),
                ("num".to_owned(), Value::Function(
                    "num".to_owned(),
                    FunctionArguments::new(Vec::from([FunctionArgument::Required("any".to_string())])), 
                    FuncImpl::Builtin(|vals| {
                        Value::Number(vals.get("any").unwrap().as_number())
                    })
                )),
                ("bool".to_owned(), Value::Function(
                    "bool".to_owned(),
                    FunctionArguments::new(Vec::from([FunctionArgument::Required("any".to_string())])), 
                    FuncImpl::Builtin(|vals| {
                        Value::Boolean(vals.get("any").unwrap().as_bool())
                    })
                )),
                ("str".to_owned(), Value::Function(
                    "str".to_owned(),
                    FunctionArguments::new(Vec::from([FunctionArgument::Required("any".to_string())])), 
                    FuncImpl::Builtin(|vals| {
                        Value::String(vals.get("any").unwrap().as_string())
                    })
                )),
            ]),
            filename
        }
    }

    pub fn get(&self, name: String) -> &Value {
        let scope = self.find_scope(name.clone());
        
        scope.variables.get(&name).unwrap_or(&Value::Null)
    }

    pub fn set(&mut self, name: String, value: Value) -> Value {
        self.variables.insert(name, value).unwrap_or(Value::Null)
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

    pub fn throw_exception(&self, msg: String, pos: Vec<usize>) {
        let pos = pos.iter().map(|u| (*u as i64).to_string()).collect::<Vec<String>>();
        println!("{}: {}\n     at: {}:{}", "ERR".bold().red(), msg, self.filename, &pos.join(":"));
        exit(-1)
    }
}