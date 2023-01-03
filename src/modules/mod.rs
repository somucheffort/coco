use phf::phf_map;

use crate::interpreter::scope::Scope;

use self::io::IOModule;

pub mod io;

pub trait CocoModule {
    fn init(scope: &mut Scope, objects: Option<Vec<String>>);
}

pub fn import_module(module: &str, scope: &mut Scope, objects: Option<Vec<String>>) {
    match module {
        "io" => {
            IOModule::init(scope, objects);
        },
        _ => panic!("Unknown module")
    }
}