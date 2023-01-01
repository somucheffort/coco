

extern crate phf;
extern crate lazy_static;

pub mod lexer;
pub mod parser;
pub mod interpreter;

use lexer::{ Lexer };
use parser::{ Parser };
use interpreter::{ scope::Scope, Interpreter };


fn main() {
    let mut lexer = Lexer::new("
    let a = log(2)
    ");
    lexer.analyse();

    let mut parser = Parser::new(lexer.tokens);
    let parsed = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let mut scope: Scope = Scope::new(None); 
    let result = interpreter.interpret(parsed, &mut scope);

    println!("{:#?}", result);
    println!("{:#?}", scope);

    //println!("{:#?}", parsed.unwrap());
}
