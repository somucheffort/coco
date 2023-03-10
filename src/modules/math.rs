use std::{collections::BTreeMap};
use rand::{ thread_rng, Rng };

use crate::interpreter::{scope::{Scope}, types::{Value, FuncImpl, FieldAccessor, FunctionArguments, FunctionArgument}};

use super::CocoModule;

pub struct MathModule {}

impl CocoModule for MathModule {
    fn init(scope: &mut Scope, objects: Option<Vec<String>>) {
        let math = get_math();

        if let Some(objects_some) = objects {
            for obj in objects_some.iter() {
                let mut field_accessor = FieldAccessor::new(math.clone(), Vec::from([Value::String(obj.to_string())]));
                let value = field_accessor.get(scope);
                scope.set(obj.to_string(), value);
            }
            return
        }
        scope.set("math".to_string(), math);
    }
}

fn get_math() -> Value {
    Value::Object(
        BTreeMap::from([ 
            ("pow".to_string(), Box::new(get_pow())),
            ("abs".to_string(), Box::new(get_abs())),
            ("ceil".to_string(), Box::new(get_ceil())),
            ("floor".to_string(), Box::new(get_floor())),
            ("round".to_string(), Box::new(get_round())),
            ("random".to_string(), Box::new(get_random())),
            ("max".to_string(), Box::new(get_max())),
            ("min".to_string(), Box::new(get_min()))
        ])
    )
}

fn get_pow() -> Value {
    Value::Function(
        "pow".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Required("num".to_string()), FunctionArgument::Required("pow".to_string())])),
        FuncImpl::Builtin(|args| {
            Value::Number(args.get("num").unwrap().as_number().powf(args.get("pow").unwrap().as_number()))
        }
    ))
}

fn get_abs() -> Value {
    Value::Function(
        "abs".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Required("num".to_string())])),
        FuncImpl::Builtin(|args| {
            Value::Number(args.get("num").unwrap().as_number().abs())
        }
    ))
}

fn get_ceil() -> Value {
    Value::Function(
        "ceil".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Required("num".to_string())])),
        FuncImpl::Builtin(|args| {
            Value::Number(args.get("num").unwrap().as_number().ceil())
        }
    ))
}

fn get_floor() -> Value {
    Value::Function(
        "floor".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Required("num".to_string())])),
        FuncImpl::Builtin(|args| {
            Value::Number(args.get("num").unwrap().as_number().floor())
        }
    ))
}

fn get_round() -> Value {
    Value::Function(
        "round".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Required("num".to_string())])),
        FuncImpl::Builtin(|args| {
            Value::Number(args.get("num").unwrap().as_number().round())
        }
    ))
}

fn get_random() -> Value {
    Value::Function(
        "random".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Spread("".to_string())])), 
        FuncImpl::Builtin(|_| {
            let mut rng = thread_rng();
            Value::Number(rng.gen())
        }
    ))
}

fn get_max() -> Value {
    Value::Function(
        "max".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Required("num1".to_string()), FunctionArgument::Required("num2".to_string())])), 
        FuncImpl::Builtin(|args| {
            args
            .into_values()
            .into_iter()
            .max_by(|v1, v2| v1.as_number().total_cmp(&v2.as_number()))
            .unwrap_or(Value::Null)
        }
    ))
}

fn get_min() -> Value {
    Value::Function(
        "min".to_owned(),
        FunctionArguments::new(Vec::from([FunctionArgument::Required("num1".to_string()), FunctionArgument::Required("num2".to_string())])), 
        FuncImpl::Builtin(|args| {
            args
            .into_values()
            .into_iter()
            .min_by(|v1, v2| v1.as_number().total_cmp(&v2.as_number()))
            .unwrap_or(Value::Null)
        }
    ))
}