

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
    fun power(a, b) {
        return a ** b
    }

    fun sum(a, b) {
        return a + b
    }

    let a = 12
    let b = 3

    log(power(power(a, b), sum(a, b)))
    ");
    lexer.analyse();

    let mut parser = Parser::new(lexer.tokens);
    let parsed = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let mut scope: Scope = Scope::new(None); 
    let result = interpreter.interpret(parsed, &mut scope);

    //println!("{:#?}", result);
    //println!("{:#?}", scope);

    //println!("{:#?}", parsed.unwrap());
}
