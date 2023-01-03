use std::{collections::BTreeMap, io::{ self, Write }};
use rand::{ thread_rng, Rng };

use crate::interpreter::{scope::Scope, types::{CocoValue, Fun, FieldAccessor}};

use super::CocoModule;

pub struct MathModule {}

impl CocoModule for MathModule {
    fn init(scope: &mut Scope, objects: Option<Vec<String>>) {
        let math = get_math();
        if let Some(objects_some) = objects {
            for obj in objects_some.iter() {
                let field_accessor = FieldAccessor::new(math.clone(), Vec::from([CocoValue::CocoString(obj.to_string())]));
                scope.set(obj.to_string(), field_accessor.get());
            }
            return
        }
        scope.set("math".to_string(), math);
    }
}

fn get_math() -> CocoValue {
    CocoValue::CocoObject(
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

fn get_pow() -> CocoValue {
    CocoValue::CocoFunction(Vec::from(["num".to_string(), "power".to_string()]), Fun::Builtin(|args| {
        CocoValue::CocoNumber(args[0].as_number().powf(args[1].as_number()))
    }))
}

fn get_abs() -> CocoValue {
    CocoValue::CocoFunction(Vec::from(["num".to_string()]), Fun::Builtin(|args| {
        CocoValue::CocoNumber(args[0].as_number().abs())
    }))
}

fn get_ceil() -> CocoValue {
    CocoValue::CocoFunction(Vec::from(["num".to_string()]), Fun::Builtin(|args| {
        CocoValue::CocoNumber(args[0].as_number().ceil())
    }))
}

fn get_floor() -> CocoValue {
    CocoValue::CocoFunction(Vec::from(["num".to_string()]), Fun::Builtin(|args| {
        CocoValue::CocoNumber(args[0].as_number().floor())
    }))
}

fn get_round() -> CocoValue {
    CocoValue::CocoFunction(Vec::from(["num".to_string()]), Fun::Builtin(|args| {
        CocoValue::CocoNumber(args[0].as_number().round())
    }))
}

fn get_random() -> CocoValue {
    CocoValue::CocoFunction(Vec::from([]), Fun::Builtin(|args| {
        let mut rng = thread_rng();
        CocoValue::CocoNumber(rng.gen())
    }))
}

fn get_max() -> CocoValue {
    CocoValue::CocoFunction(Vec::from(["num".to_string()]), Fun::Builtin(|args| {
        args
        .iter()
        .max_by(|v1, v2| v1.as_number().total_cmp(&v2.as_number()))
        .unwrap_or(&CocoValue::CocoNull)
        .to_owned()
    }))
}

fn get_min() -> CocoValue {
    CocoValue::CocoFunction(Vec::from([]), Fun::Builtin(|args| {
        args
        .iter()
        .min_by(|v1, v2| v1.as_number().total_cmp(&v2.as_number()))
        .unwrap_or(&CocoValue::CocoNull)
        .to_owned()
    }))
}