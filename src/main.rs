use std::{ fs, env, process::exit, io::{ self, Write }, };

extern crate phf;
extern crate lazy_static;

pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod modules;

use colored::Colorize;
use lexer::{ Lexer };
use parser::{ Parser };
use interpreter::{ scope::{ Scope }, walk_tree };

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    msg: String,
    pos: Vec<usize>
}

impl Error {
    pub fn exit(&self, filename: String) {
        let pos = self.pos.iter().map(|u| (*u as i64).to_string()).collect::<Vec<String>>();
        println!("{}: {}\n     at: {}:{}", "ERR".bold().red(), self.msg, filename, &pos.join(":"));
        exit(-1)
    }
}

#[derive(Debug, Clone)]
pub struct Resolver {
    filename: String,
    code: String
}

impl Resolver {
    pub fn new(filename: String, code: String) -> Self {
        Self {
            filename,
            code
        }
    } 

    pub fn resolve_where(&self, pos: usize) -> Vec<usize> {
        let lines = self.code.split('\n');
        let mut len: usize = 0;
        let mut line_start: usize = 0;
    
        for (i, line) in lines.into_iter().enumerate() {
            len += line.len() + 1;
            if pos < len {
                return vec![i + 1, pos - line_start + 1]
            }
            line_start = len;
        }
    
        vec![0, 0]
    }

    pub fn exit_error(&self, msg: String, pos: Vec<usize>) {
        Error { msg, pos }.exit(self.filename.clone()) 
    }
}

fn run_file(filename: String) {
    let input = fs::read_to_string(&filename).unwrap();

    // creating resolver for resolving position of error

    let resolver = Resolver::new(filename.clone(), input.clone());

    // getting tokens

    let mut lexer = Lexer::new(&input, &resolver);
    let tokens = lexer.analyse();

    if let Err(e) = tokens {
        e.exit(filename.to_string())
    }

    // parsing tokens in nodes

    let mut parser = Parser::new(lexer.tokens, &resolver);
    let parsed = parser.parse();

    if let Err(e) = parsed.as_ref() {
        e.exit(filename.to_string())
    }

    // executing the code
    
    let mut scope = Scope::new(filename.to_string());

    let result = walk_tree(parsed.unwrap(), &mut scope);

    //println!("{:#?}", stack);

    if let Err(e) = result {
        e.exit(filename)
    }
}

fn run_repl() {
    let filename = "<repl>".to_string();
    let mut scope = Scope::new(filename.clone());
    let resolver = Resolver::new(filename.clone(), "".to_string());

    loop {
        print!(">> ");
        let _ = io::stdout().flush();
        let mut buffer = String::new();
        if let Ok(_b) = io::stdin().read_line(&mut buffer) {   
            let mut lexer = Lexer::new(&buffer, &resolver);
            let tokens = lexer.analyse();

            if let Err(e) = tokens {
                println!("{}: {}\n     at: {}:0:0", "ERR".bold().red(), e.msg, &filename);
                return
            }

            // parsing tokens in nodes

            let mut parser = Parser::new(lexer.tokens, &resolver);
            let parsed = parser.parse();

            if let Err(e) = parsed.as_ref() {
                println!("{}: {}\n     at: {}:0:0", "ERR".bold().red(), e.msg, &filename);
                return
            }

            let result = walk_tree(parsed.unwrap(), &mut scope);

            // print!("{buffer}")
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        //println!("{}: Expected filename, e.g. \n     {}", "ERR".bold().red(), "coco filename.co".bold());
        //exit(-1)

        run_repl()
    }

    let filename = &args[1];
    run_file(filename.to_owned());
}
