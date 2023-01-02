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
    // CocoObject
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
            CocoValue::CocoNull => 0.0
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            CocoValue::CocoString(val) => val.to_owned(),
            CocoValue::CocoNumber(val) => val.to_string(),
            CocoValue::CocoBoolean(val) => val.to_string(),
            CocoValue::CocoArray(values) => {
                let mut str = "".to_owned();
                let mut iter = values.iter();

                loop {
                    let val = iter.next().unwrap();
                    str.push_str(&val.as_string());
                    if iter.next().is_some() {
                        str.push(',')
                    }
                } 

                str
            },
            CocoValue::CocoFunction(_s, _n) => "NotImplemented".to_owned(),
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
                            let rev: Vec<&Box<CocoValue>> = array.into_iter().rev().collect();

                            return *rev.get(val.abs() as usize).unwrap_or(&&Box::new(CocoValue::CocoNull)).to_owned().to_owned()
                        }

                        *array.get(val as usize).unwrap_or(&Box::new(CocoValue::CocoNull)).to_owned()
                    },
                    _ => panic!("Expected number or string")
                }
            },
            _ => CocoValue::CocoNull,
        }
    }
}

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
            _ => panic!("Array or string expected")
        }
    }

    pub fn get_container(&self) -> CocoValue {
        let mut container = self.value.clone();
        for i in 0..self.fields.len() - 1 {
            match self.value.clone() {
                CocoValue::CocoArray(val) => {
                    container = self.value.get_field(self.fields.get(i).unwrap().to_owned())
                },
                _ => panic!("Array expected"),
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
            CocoValue::CocoArray(values) => write!(f, "{:#?}", values.iter().map(|x| *x.to_owned()).map(|x| x.to_string()).collect::<Vec<_>>()),
            CocoValue::CocoFunction(_s, _n) => write!(f, "{:#?} {:#?}", _s, _n),
            CocoValue::CocoNull => write!(f, "null")
        }
    }
}