use std::{collections::{BTreeMap, HashMap}, cmp::Ordering};

use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;

use crate::parser::Node;

use super::{scope::Scope, walk_tree};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum FuncImpl {
    FromNode(Node),
    Builtin(fn(HashMap<String, CocoValue>) -> CocoValue)
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum FuncArg {
    Required(String),
    NotRequired(String, CocoValue),
    Spread(String)
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FuncArgs {
    args: Vec<FuncArg>
}

impl FuncArgs {
    pub fn new(args: Vec<FuncArg>) -> Self { 
        Self {
            args
        }
    }

    pub fn add_argument(&mut self, arg: FuncArg) {
        self.args.push(arg)
    }

    pub fn reduce(&mut self, args_eval: &mut Vec<CocoValue>) -> HashMap<String, CocoValue> {
        args_eval.reverse();
        self.args.clone().into_iter().fold(HashMap::default(), | mut acc, value | {
            match value {
                FuncArg::Required(name) => {
                    acc.insert(name, args_eval.pop().unwrap());
                    acc
                },
                FuncArg::NotRequired(name, value) => {
                    let current_val = args_eval.pop();
                    acc.insert(name, current_val.unwrap_or(value));
                    acc
                },
                FuncArg::Spread(name) => {
                    let mut spreaded = args_eval.clone();
                    spreaded.reverse();
                    acc.insert(name, CocoValue::CocoArray(
                        spreaded.iter().map(|v| Box::new(v.to_owned())).collect::<Vec<Box<CocoValue>>>()
                    ));
                    acc
                }
            }
        })
    }

    pub fn get_arguments(&self) -> Vec<FuncArg> {
        self.args.clone()
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum CocoValue {
    CocoString(String),
    CocoNumber(f64),
    CocoBoolean(bool),
    CocoArray(Vec<Box<CocoValue>>),
    CocoObject(BTreeMap<String, Box<CocoValue>>),
    // FIXME: args
    CocoFunction(FuncArgs, FuncImpl),
    // CocoClass
    CocoNull
}

pub fn create_string(s: String, scope: &mut Scope) -> CocoValue {
    lazy_static! {
        static ref VAR_REGEX: Regex = Regex::new(r"\$([a-zA-Z][0-9a-zA-Z_]*)").unwrap();
    }

    let mut new_string = s;

    let values = VAR_REGEX.find_iter(new_string.as_str()).map(|s| s.as_str().to_string()).collect::<Vec<String>>();
    for value in values.iter() {
        new_string = new_string.replace(value, &scope.get(value.to_string().replace('$', "")).as_string());
    }

    CocoValue::CocoString(new_string)
}

impl CocoValue {
    pub fn as_bool(&self) -> bool {
        match self {
            CocoValue::CocoString(val) => !val.is_empty(),
            CocoValue::CocoNumber(val) => *val as i64 == 0,
            CocoValue::CocoBoolean(val) => *val,
            CocoValue::CocoArray(values) => !values.is_empty(),
            CocoValue::CocoFunction(_s, _n) => true,
            CocoValue::CocoObject(map) => !map.is_empty(),
            CocoValue::CocoNull => false
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            CocoValue::CocoString(val) => val.parse::<f64>().unwrap_or(f64::NAN),
            CocoValue::CocoNumber(val) => *val,
            CocoValue::CocoBoolean(val) => *val as i64 as f64,
            CocoValue::CocoArray(_values) => f64::NAN,
            CocoValue::CocoFunction(_s, _n) => f64::NAN,
            CocoValue::CocoObject(_map) => f64::NAN,
            CocoValue::CocoNull => 0.0
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            CocoValue::CocoString(val) => val.to_owned(),
            CocoValue::CocoNumber(val) => val.to_string(),
            CocoValue::CocoBoolean(val) => val.to_string(),
            CocoValue::CocoArray(values) => values.iter().map(|x| x.as_string()).collect::<Vec<_>>().join(","),
            CocoValue::CocoFunction(_s, _n) => "NotImplemented".to_owned(),
            CocoValue::CocoObject(map) => map.iter()
            .map(|x| (x.0, *x.1.to_owned()))
            .map(|x| format!("{}: {}", x.0, x.1.as_string()))
            .collect::<Vec<_>>().join(", "),
            CocoValue::CocoNull => "null".to_owned()
        }
    }

    pub fn compare(&self, value: CocoValue) -> Ordering {
        match self {
            CocoValue::CocoString(val) => val.cmp(&value.as_string()),
            CocoValue::CocoNumber(val) => val.total_cmp(&value.as_number()),
            CocoValue::CocoBoolean(val) => val.cmp(&value.as_bool()),
            CocoValue::CocoArray(_values) => self.partial_cmp(&value).unwrap(),
            CocoValue::CocoFunction(_s, _n) => self.partial_cmp(&value).unwrap(),
            CocoValue::CocoObject(_map) => self.partial_cmp(&value).unwrap(),
            CocoValue::CocoNull => self.partial_cmp(&value).unwrap()
        }
    }

    pub fn get_field(&mut self, field: CocoValue, scope: &mut Scope) -> CocoValue {
        match self {
            CocoValue::CocoString(string) => {
                match field {
                    CocoValue::CocoString(val) => {
                        match val.as_str() {
                            "length" => CocoValue::CocoNumber(string.len() as f64),
                            _ => CocoValue::CocoNull
                        }
                    },
                    CocoValue::CocoNumber(val) => {
                        // FIXME
                        if val < 0.0 {
                            let rev = string.chars().rev().collect::<String>();

                            let val_abs = val.abs() as usize;

                            return CocoValue::CocoString(rev.get(val_abs..val_abs+1).unwrap().to_string())
                        }

                        let val_usize = val as usize;

                        CocoValue::CocoString(string.get(val_usize..val_usize+1).unwrap().to_string())
                    },
                    _ => panic!("Expected number or string")
                }
            },
            CocoValue::CocoArray(array) => {
                match field {
                    CocoValue::CocoString(val) => {
                        match val.as_str() {
                            "length" => CocoValue::CocoNumber(array.len() as f64),
                            _ => CocoValue::CocoNull
                        }
                    },
                    CocoValue::CocoNumber(val) => {
                        if val.is_sign_negative() {
                            let len = array.len() as f64;
                            return *array.get((len + val) as usize).unwrap_or(&Box::new(CocoValue::CocoNull)).to_owned()
                        }

                        *array.get(val as usize).unwrap_or(&Box::new(CocoValue::CocoNull)).to_owned()
                    },
                    _ => panic!("Expected number or string")
                }
            },
            CocoValue::CocoObject(map) => {
                match field {
                    CocoValue::CocoString(val) => {
                        *map.to_owned().get(&val).unwrap_or(&Box::new(CocoValue::CocoNull)).to_owned()
                    },
                    // FIXME
                    _ => panic!("Unknown field")
                }
            },
            _ => CocoValue::CocoNull,
        }
    }

    pub fn set_field(&mut self, field: CocoValue, value: CocoValue) -> CocoValue {
        match self {
            CocoValue::CocoArray(array) => {
                match field {
                    CocoValue::CocoNumber(val) => {
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
            CocoValue::CocoObject(map) => {
                if let CocoValue::CocoString(val) = field {
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
    value: CocoValue,
    fields: Vec<CocoValue>
}

impl FieldAccessor {
    pub fn new(value: CocoValue, fields: Vec<CocoValue>) -> Self {
        Self { value, fields }
    }

    pub fn get(&mut self, scope: &mut Scope) -> CocoValue {
        let mut container = self.get_container(scope);
        let last = self.last();

        match container.clone() {
            CocoValue::CocoString(_val) => container.get_field(last, scope),
            CocoValue::CocoArray(_vals) => container.get_field(last, scope),
            CocoValue::CocoObject(_vals) => container.get_field(last, scope),
            _ => panic!("Array, string or object expected")
        }
    }

    pub fn set(&mut self, value: CocoValue, scope: &mut Scope) -> CocoValue {
        let mut container = self.get_container(scope);
        let last = self.last();

        match container.clone() {
            CocoValue::CocoArray(_vals) => container.set_field(last, value),
            CocoValue::CocoObject(_vals) => container.set_field(last, value),
            _ => panic!("Array or object expected")
        }
    }

    pub fn get_container(&mut self, scope: &mut Scope) -> CocoValue {
        let mut container = self.value.clone();
        for i in 0..self.fields.len() - 1 {
            match self.value.clone() {
                CocoValue::CocoArray(_val) => {
                    container = self.value.get_field(self.fields.get(i).unwrap().to_owned(), scope)
                },
                CocoValue::CocoObject(_val) => {
                    container = self.value.get_field(self.fields.get(i).unwrap().to_owned(), scope)
                },
                _ => panic!("Array or object expected"),
            }
        }

        container
    }

    pub fn last(&self) -> CocoValue {
        self.fields.last().unwrap_or(&CocoValue::CocoNull).to_owned()
    }

    pub fn by_index(&self, i: usize) -> CocoValue {
        self.fields.get(i).unwrap_or(&CocoValue::CocoNull).to_owned()
    }
}

impl std::fmt::Display for CocoValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {
            CocoValue::CocoString(_val) => write!(f, "{}", ("'".to_owned() + &self.as_string() + "'").green()),
            CocoValue::CocoNumber(_val) => write!(f, "{}", &self.as_string().yellow()),
            CocoValue::CocoBoolean(_val) => write!(f, "{}", &self.as_string().blue()),
            CocoValue::CocoArray(values) => write!(f, "[ {} ]", values.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
            CocoValue::CocoFunction(_s, _n) => write!(f, "{:#?} {:#?}", _s, _n),
            CocoValue::CocoObject(_map) => write!(f, "{{ {} }}", &self.as_string()),
            CocoValue::CocoNull => write!(f, "{}", "null".bold())
        }
    }
}