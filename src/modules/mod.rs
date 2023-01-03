use crate::interpreter::scope::Scope;

use self::{io::IOModule, math::MathModule};

pub mod io;
pub mod math;

pub trait CocoModule {
    fn init(scope: &mut Scope, objects: Option<Vec<String>>);
}

pub fn import_module(module: &str, scope: &mut Scope, objects: Option<Vec<String>>) {
    //println!("{}", module);
    match module {
        "io" => IOModule::init(scope, objects),
        "math" => MathModule::init(scope, objects),
        _ => panic!("Unknown module")
    }
}