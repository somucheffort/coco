use std::{ fs, env };

extern crate phf;
extern crate lazy_static;

pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod modules;

use colored::Colorize;
use lexer::{ Lexer };
use parser::{ Parser };
use interpreter::{ scope::Scope, Interpreter };


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("{}: Expected filename, e.g. \n     {}", "ERR".bold().red(), "coco filename.co".bold());
        return
    }

    let input = fs::read_to_string(&args[1]).unwrap();

    let mut lexer = Lexer::new(&input);
    lexer.analyse();

    let mut parser = Parser::new(lexer.tokens);
    let parsed = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    let mut scope: Scope = Scope::new(None);
    interpreter.interpret(parsed, &mut scope);
}
