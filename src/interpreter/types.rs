use crate::parser::Node;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum CocoValue {
    CocoString(String),
    CocoNumber(f64),
    CocoBoolean(bool),
    CocoArray(Vec<Box<CocoValue>>),
    // CocoObject
    // FIXME: args
    CocoFunction(Vec<String>, Node),
    // CocoClass
    CocoNull
}

impl CocoValue {
    pub fn as_bool(&self) -> bool {
        match self {
            CocoValue::CocoString(val) => val.len() > 0,
            CocoValue::CocoNumber(val) => *val as i64 == 0,
            CocoValue::CocoBoolean(val) => *val,
            CocoValue::CocoArray(values) => values.len() > 0,
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
            CocoValue::CocoNumber(val) => stringify!(*val).to_string(),
            CocoValue::CocoBoolean(val) => stringify!(*val).to_string(),
            CocoValue::CocoArray(values) => {
                let mut str = "".to_owned();
                let mut iter = values.iter();

                loop {
                    let val = iter.next();
                    str.push_str(&val.unwrap().as_string());
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
}

impl std::fmt::Display for CocoValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CocoValue::CocoString(val) => write!(f, "{}", val),
            CocoValue::CocoNumber(val) => write!(f, "{}", val),
            CocoValue::CocoBoolean(val) => write!(f, "{}", val),
            CocoValue::CocoArray(_values) => write!(f, "{:#?}", _values),
            CocoValue::CocoFunction(_s, _n) => write!(f, "{:#?} {:#?}", _s, _n),
            CocoValue::CocoNull => write!(f, "null")
        }
    }
}