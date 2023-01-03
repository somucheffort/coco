use std::collections::BTreeMap;

use colored::Colorize;

use crate::parser::Node;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Fun {
    Node(Node),
    Builtin(fn(Vec<CocoValue>) -> CocoValue)
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum CocoValue {
    CocoString(String),
    CocoNumber(f64),
    CocoBoolean(bool),
    CocoArray(Vec<Box<CocoValue>>),
    CocoObject(BTreeMap<String, Box<CocoValue>>),
    // FIXME: args
    CocoFunction(Vec<String>, Fun),
    // CocoClass
    CocoNull
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

    pub fn get_field(&self, field: CocoValue) -> CocoValue {
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
                        // FIXME
                        if val < 0.0 {
                            let rev: Vec<&Box<CocoValue>> = array.iter().rev().collect();

                            // FIXME: revisit it
                            return *rev.get(val.abs() as usize).unwrap_or(&&Box::new(CocoValue::CocoNull)).to_owned().to_owned()
                        }

                        *array.get(val as usize).unwrap_or(&Box::new(CocoValue::CocoNull)).to_owned()
                    },
                    _ => panic!("Expected number or string")
                }
            },
            CocoValue::CocoObject(map) => {
                match field {
                    CocoValue::CocoString(val) => {
                        *map.get(&val).unwrap_or(&Box::new(CocoValue::CocoNull)).to_owned()
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
                        // FIXME 
                        /*
                        if val < 0.0 {
                            let rev: Vec<&Box<CocoValue>> = array.into_iter().rev().collect();

                            // FIXME: revisit it
                            return *rev.get(val.abs() as usize).unwrap_or(&&Box::new(CocoValue::CocoNull)).to_owned().to_owned()
                        }*/

                        *array[val as usize] = value;

                        CocoValue::CocoArray(array.to_vec())
                    },
                    _ => panic!("Expected number")
                }
            },
            CocoValue::CocoObject(map) => {
                match field {
                    CocoValue::CocoString(val) => {
                        let mut new_map = map.to_owned();
                        new_map.insert(val, Box::new(value));

                        CocoValue::CocoObject(new_map)
                    },
                    // FIXME
                    _ => panic!("Unknown field")
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

    pub fn get(&self) -> CocoValue {
        let container = self.get_container();
        let last = self.last();

        match container.clone() {
            CocoValue::CocoString(_val) => container.get_field(last),
            CocoValue::CocoArray(_vals) => container.get_field(last),
            CocoValue::CocoObject(_vals) => container.get_field(last),
            _ => panic!("Array, string or object expected")
        }
    }

    pub fn set(&mut self, value: CocoValue) -> CocoValue {
        let mut container = self.get_container();
        let last = self.last();

        match container.clone() {
            CocoValue::CocoArray(_vals) => container.set_field(last, value),
            CocoValue::CocoObject(_vals) => container.set_field(last, value),
            _ => panic!("Array or object expected")
        }
    }

    pub fn get_container(&self) -> CocoValue {
        let mut container = self.value.clone();
        for i in 0..self.fields.len() - 1 {
            match self.value.clone() {
                CocoValue::CocoArray(_val) => {
                    container = self.value.get_field(self.fields.get(i).unwrap().to_owned())
                },
                CocoValue::CocoObject(_val) => {
                    container = self.value.get_field(self.fields.get(i).unwrap().to_owned())
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
            CocoValue::CocoObject(map) => write!(f, "{{ {} }}", &self.as_string()),
            CocoValue::CocoNull => write!(f, "{}", "null".bold())
        }
    }
}