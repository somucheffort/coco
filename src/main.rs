use std::{ fs, env };

extern crate phf;
extern crate lazy_static;

pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod modules;

use lexer::{ Lexer };
use parser::{ Parser };
use interpreter::{ scope::Scope, Interpreter };


fn main() {
    let args: Vec<String> = env::args().collect();

    let mut input = "
    
    ".to_string();

    if args.len() > 1 {
        input = fs::read_to_string(&args[1]).unwrap_or_else(|_| input.to_string());
    }

    let mut lexer = Lexer::new(&input);

    lexer.analyse();

    //println!("{:#?}", lexer);

    let mut parser = Parser::new(lexer.tokens);
    let parsed = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let mut scope: Scope = Scope::new(None);
    let result = interpreter.interpret(parsed, &mut scope);

    //println!("{:#?}", result);
    //println!("{:#?}", scope);

    //println!("{:#?}", parsed.unwrap());
}
