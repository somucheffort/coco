use std::{collections::{BTreeMap, HashMap}, cmp::Ordering};

use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;

use crate::parser::Node;

use super::{scope::{Scope}, walk_tree};



lazy_static! {
    static ref VAR_REGEX: Regex = Regex::new(r"\$([a-zA-Z][0-9a-zA-Z_]*)").unwrap();
}



#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum FuncImpl {
    FromNode(Node),
    Builtin(fn(HashMap<String, Value>) -> Value)
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum FunctionArgument {
    Required(String),
    NotRequired(String, Value),
    Spread(String)
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FunctionArguments {
    args: Vec<FunctionArgument>
}

impl FunctionArguments {
    pub fn new(args: Vec<FunctionArgument>) -> Self { 
        Self {
            args
        }
    }

    pub fn add(&mut self, arg: FunctionArgument) {
        self.args.push(arg)
    }

    pub fn get(&self) -> Vec<FunctionArgument> {
        self.args.clone()
    }

    pub fn reduce(&mut self, args_eval: &mut Vec<Value>) -> HashMap<String, Value> {
        args_eval.reverse();
        self.args.clone().into_iter().fold(HashMap::default(), | mut acc, value | {
            match value {
                FunctionArgument::Required(name) => {
                    acc.insert(name, args_eval.pop().unwrap());
                    acc
                },
                FunctionArgument::NotRequired(name, value) => {
                    let current_val = args_eval.pop();
                    acc.insert(name, current_val.unwrap_or(value));
                    acc
                },
                FunctionArgument::Spread(name) => {
                    let mut spreaded = args_eval.clone();
                    spreaded.reverse();
                    acc.insert(name, Value::Array(
                        spreaded.iter().map(|v| Box::new(v.to_owned())).collect::<Vec<Box<Value>>>()
                    ));
                    acc
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Box<Value>>),
    Object(BTreeMap<String, Box<Value>>),
    Function(String, FunctionArguments, FuncImpl),
    Class(String, Option<Box<Value>>, BTreeMap<String, Box<Value>>),
    Null
}

impl Value {
    pub fn create_string(s: String, scope: &mut Scope) -> Value {
        let mut new_string = s;

        let variables = VAR_REGEX.find_iter(new_string.as_str()).map(|s| s.as_str().to_string()).collect::<Vec<String>>();
        for variable in variables.iter() {
            let value = scope.get(variable.to_string().replace('$', ""));
            new_string = new_string.replace(variable, &value.as_string());
        }

        Value::String(new_string)
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::String(val) => !val.is_empty(),
            Value::Number(val) => *val as i64 == 0,
            Value::Boolean(val) => *val,
            Value::Array(values) => !values.is_empty(),
            Value::Function(_n, _a, _i) => true,
            Value::Object(map) => !map.is_empty(),
            Value::Null => false,
            Value::Class(_n, _p, _c) => true
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            Value::String(val) => val.parse::<f64>().unwrap_or(f64::NAN),
            Value::Number(val) => *val,
            Value::Boolean(val) => *val as i64 as f64,
            Value::Array(_values) => f64::NAN,
            Value::Function(_n, _a, _i) => f64::NAN,
            Value::Object(_map) => f64::NAN,
            Value::Null => 0.0,
            Value::Class(_n, _p, _c) => f64::NAN
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Value::String(val) => val.to_owned(),
            Value::Number(val) => val.to_string(),
            Value::Boolean(val) => val.to_string(),
            Value::Array(values) => values.iter().map(|x| x.as_string()).collect::<Vec<_>>().join(","),
            Value::Function(name, _s, _n) => format!("fun {} {{ ... }}", name),
            Value::Object(map) => map.iter()
            .map(|x| (x.0, *x.1.to_owned()))
            .map(|x| format!("{}: {}", x.0, x.1.as_string()))
            .collect::<Vec<_>>().join(", "),
            Value::Null => "null".to_owned(),
            Value::Class(name, _p, _c) => format!("class {} {{ ... }}", name)
        }
    }

    pub fn compare(&self, value: Value) -> Ordering {
        match self {
            Value::String(val) => val.cmp(&value.as_string()),
            Value::Number(val) => val.total_cmp(&value.as_number()),
            Value::Boolean(val) => val.cmp(&value.as_bool()),
            Value::Array(_values) => self.partial_cmp(&value).unwrap(),
            Value::Function(_n, _a, _i) => self.partial_cmp(&value).unwrap(),
            Value::Object(_map) => self.partial_cmp(&value).unwrap(),
            Value::Null => self.partial_cmp(&value).unwrap(),
            Value::Class(_n, _p, _c) => self.partial_cmp(&value).unwrap()
        }
    }

    pub fn get_field(&mut self, field: Value, scope: &mut Scope) -> Value {
        match self {
            Value::String(string) => {
                match field {
                    Value::String(val) => {
                        match val.as_str() {
                            "length" => Value::Number(string.len() as f64),
                            _ => Value::Null
                        }
                    },
                    Value::Number(val) => {
                        if val.is_sign_negative() {
                            string.reverse();
                        }

                        let index = val.abs() as usize;

                        Value::String(string.get(index..index+1).unwrap().to_string())
                    },
                    _ => panic!("Expected number or string")
                }
            },
            Value::Array(array) => {
                match field {
                    Value::String(val) => {
                        match val.as_str() {
                            "length" => Value::Number(array.len() as f64),
                            _ => Value::Null
                        }
                    },
                    Value::Number(mut val) => {
                        if val.is_sign_negative() {
                            val += array.len() as f64;    
                        }

                        *array.get(val as usize).unwrap_or(&Box::new(Value::Null)).to_owned()
                    },
                    _ => {
                        scope.throw_exception("Expected number or string".to_string(), vec![0,0]);
                        Value::Null
                    }
                }
            },
            Value::Object(map) => {
                match field {
                    Value::String(val) => {
                        *map.to_owned().get(&val).unwrap_or(&Box::new(Value::Null)).to_owned()
                    },
                    // FIXME
                    _ => {
                        scope.throw_exception("Unknown field".to_string(), vec![0,0]);
                        Value::Null
                    }
                }
            },
            _ => Value::Null,
        }
    }

    pub fn set_field(&mut self, field: Value, value: Value) -> Value {
        match self {
            Value::Array(array) => {
                match field {
                    Value::Number(val) => {
                        if val.is_sign_negative() {
                            let len = array.len() as f64;
                            array[(len + val) as usize] = Box::new(value);
                        } else {
                            array[val as usize] = Box::new(value);
                        }

                        self.to_owned()
                    },
                    _ => panic!("Expected number")
                }
            },
            Value::Object(map) => {
                if let Value::String(val) = field {
                    map.insert(val, Box::new(value));

                    self.to_owned()
                } else {
                    panic!("Unknown field")
                }
            },

            // FIXME
            _ => panic!("Cannot set field to this value")
        }
    }
}

#[derive(Debug)]
pub struct FieldAccessor {
    value: Value,
    fields: Vec<Value>
}

impl FieldAccessor {
    pub fn new(value: Value, fields: Vec<Value>) -> Self {
        Self { value, fields }
    }

    pub fn get(&mut self, scope: &mut Scope) -> Value {
        let mut container = self.get_container(scope);
        let last = self.last();

        match container.clone() {
            Value::String(_val) => container.get_field(last, scope),
            Value::Array(_vals) => container.get_field(last, scope),
            Value::Object(_vals) => container.get_field(last, scope),
            _ => panic!("Array, string or object expected")
        }
    }

    pub fn set(&mut self, value: Value, scope: &mut Scope) -> Value {
        let mut container = self.get_container(scope);
        let last = self.last();

        match container.clone() {
            Value::Array(_vals) => container.set_field(last, value),
            Value::Object(_vals) => container.set_field(last, value),
            _ => panic!("Array or object expected")
        }
    }

    pub fn get_container(&mut self, scope: &mut Scope) -> Value {
        let mut container = self.value.clone();
        for i in 0..self.fields.len() - 1 {
            match self.value.clone() {
                Value::Array(_val) => {
                    container = self.value.get_field(self.fields.get(i).unwrap().to_owned(), scope)
                },
                Value::Object(_val) => {
                    container = self.value.get_field(self.fields.get(i).unwrap().to_owned(), scope)
                },
                _ => panic!("Array or object expected"),
            }
        }

        container
    }

    pub fn last(&self) -> Value {
        self.fields.last().unwrap_or(&Value::Null).to_owned()
    }

    pub fn by_index(&self, i: usize) -> Value {
        self.fields.get(i).unwrap_or(&Value::Null).to_owned()
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {
            Value::String(_val) => write!(f, "{}", ("'".to_owned() + &self.as_string() + "'").green()),
            Value::Number(_val) => write!(f, "{}", &self.as_string().yellow()),
            Value::Boolean(_val) => write!(f, "{}", &self.as_string().blue()),
            Value::Array(values) => write!(f, "[ {} ]", values.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
            Value::Function(name, _a, _i) => write!(f, "fun {} {{ ... }}", name),
            Value::Object(_map) => write!(f, "{{ {} }}", &self.as_string()),
            Value::Null => write!(f, "{}", "null".bold()),
            Value::Class(name, _p, _c) => write!(f, "class {} {{ ... }}", name),
        }
    }
}