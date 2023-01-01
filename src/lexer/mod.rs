use std::{io::Error, io::ErrorKind};
use phf::{ phf_map };

const QUOTES: &str = "\'\"";
const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &str = "0123456789";
//const VARIABLE_REGEX: Regex = Regex::new("[a-zA-Z]+\\d*");

const KEYWORDS: phf::Map<&str, TokenType> = phf_map! {
    "let" => TokenType::LET,
    "fun" =>  TokenType::FUN,
    "return" =>  TokenType::RETURN,
    "if" =>  TokenType::IF,
    "else" => TokenType::ELSE,
    "true" =>  TokenType::BOOLEAN,
    "false" =>  TokenType::BOOLEAN,
    "for" =>  TokenType::FOR,
    "in" =>  TokenType::IN,
    "switch" =>  TokenType::SWITCH,
    "case" =>  TokenType::CASE,
    "default" =>  TokenType::DEFAULT,
    "while" =>  TokenType::WHILE,
    "do" =>  TokenType::DO,
    "break" =>  TokenType::BREAK,
    "continue" =>  TokenType::CONTINUE,
    "null" =>  TokenType::NULL,
    "typeof" => TokenType::TYPEOF,
    "class" =>  TokenType::CLASS,
    "new" =>  TokenType::NEW,
    "this" =>  TokenType::THIS
};

const OPERATORS: phf::Map<&str, TokenType> = phf_map! {
    "+" => TokenType::PLUS,
    "-" => TokenType::MINUS,
    "*" => TokenType::STAR,
    "/" => TokenType::SLASH,
    "(" => TokenType::LPAR,
    ")" => TokenType::RPAR,
    "{" => TokenType::LBRACE,
    "}" => TokenType::RBRACE,
    "[" => TokenType::LBRACKET,
    "]" => TokenType::RBRACKET,
    "=" => TokenType::EQUALS,
    "," => TokenType::COMMA,
    "!" => TokenType::EXCL,
    "==" => TokenType::EQEQ,
    "!=" => TokenType::EXCLEQ,
    ">" => TokenType::GT,
    "<" => TokenType::LT,
    "<=" => TokenType::GTEQ,
    ">=" => TokenType::LTEQ,
    "&&" => TokenType::AMPAMP,
    "||" => TokenType::BARBAR,
    "->" => TokenType::ARROW,
    "." => TokenType::DOT,
    "..." => TokenType::SPREAD,
    "?" => TokenType::QUESTION,
    ":" => TokenType::COLON,
    "**" => TokenType::DOUBLESTAR,
    "%" => TokenType::PERCENT,
    "+=" => TokenType::PLUSEQ,
    "-=" => TokenType::MINUSEQ,
    "*=" => TokenType::MULTIPLYEQ,
    "/=" => TokenType::DIVIDEEQ,
    "**=" => TokenType::EXPONENTEQ,
    "%=" => TokenType::REMAINDEREQ
};

fn is_variable(var: char) -> bool{
    LETTERS.contains(var) || DIGITS.contains(var) || var == '_'
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TokenType {
    LET, // let
    FUN, // fun
    RETURN, // return
    FOR, // for
    IN, // in
    IF, // if
    ELSE, // else
    SWITCH, // switch
    CASE, // case
    DEFAULT, // default
    WHILE, // while
    DO, // do
    BREAK, // break
    CONTINUE, // continue
    TYPEOF, // typeof
    CLASS, // class
    NEW, // new
    THIS, // this
    
    NULL, // null
    NUMBER, // 0
    STRING, // '0'
    WORD, // bones
    BOOLEAN, // true, false

    EQUALS, // =
    PLUS, // +
    MINUS, // -
    SLASH, // /
    STAR, // *
    DOUBLESTAR, // **
    PERCENT, // %
    
    PLUSEQ, // +=
    MINUSEQ, // -=
    DIVIDEEQ, // /=
    MULTIPLYEQ, // *=
    EXPONENTEQ, // **=
    REMAINDEREQ, // %=

    LPAR, // (
    RPAR, // )
    LBRACE, // {
    RBRACE, // }
    LBRACKET, // [
    RBRACKET, // ]
    COMMA, // ,
    DOT, // .
    COLON, // :
    EXCL, // !
    QUESTION, // ?
    EQEQ, // ==
    EXCLEQ, // !=
    GT, // >
    LT, // <
    GTEQ, // <=
    LTEQ, // >=
    AMPAMP, // &&
    BARBAR, // ||
    ARROW, // ->
    SPREAD, // ..

    EOF
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String
}

#[derive(Debug, Clone)]
pub struct Lexer {
    pub code: String,
    pub tokens: Vec<Token>,
    pub pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            code: input.to_owned(),
            tokens: Vec::new(),
            pos: 0
        }
    }

    pub fn analyse(&mut self) -> Vec<Token> {
        while self.pos < self.code.len() {
            let current = self.peek(None);
            let mut result = None;

            if OPERATORS.keys().find(|&&key| current.to_string().contains(key)).is_some() {
                result = Some(self.parse_operator());
            } else if DIGITS.contains(current) {
                result = Some(self.parse_number());
            } else if LETTERS.contains(current) {
                result = Some(self.parse_word());
            } else if QUOTES.contains(current) {
                result = Some(self.parse_string());
            } else  {
                self.next();
            }

            if result.is_some() {
                if result.to_owned().unwrap().is_err() {
                    panic!("{:#?}", result.unwrap().err())
                }
            }
        }

        self.tokens.clone()
    }

    pub fn parse_operator(&mut self) -> Result<bool, String> {
        let mut buffer: String = "".to_owned();
        let mut current = self.peek(None);
        loop {
            let current_buff = buffer.clone() + &current.to_string();
            if current_buff == "//" {
                return self.parse_comment(None);
            } else if current_buff == "/*" {
                return self.parse_comment(Some(true));
            }
            if OPERATORS.keys().find(|&&key| key.starts_with(current_buff.as_str())).is_none() {
                break
            }
            buffer.push(current);
            current = self.next();
        }

        self.add_token(OPERATORS.get(buffer.as_str()).unwrap().to_owned(), buffer.as_str());
        Ok(true)
    }

    pub fn parse_number(&mut self) -> Result<bool, String> {
        let mut buffer: String = "".to_owned();
        let mut current = self.peek(None);
        loop {
            if current == '.' {
                if buffer.contains('.') {
                    return Err("Invalid float".to_string())
                }
            } else if !DIGITS.contains(current) {
                break
            }
            buffer.push(current);
            current = self.next();
        }

        self.add_token(TokenType::NUMBER, buffer.as_str());

        Ok(true)
    }

    pub fn parse_string(&mut self) -> Result<bool, String> {
        let mut buffer: String = "".to_owned();
        let quote = self.peek(None);
        let mut current = self.next();

        loop {
            if current == '\0' {
                return Err("String didnt close".to_string());
            }
            if current == quote {
                break;
            }
            buffer.push(current);
            current = self.next();
        }

        self.next();
        self.add_token(TokenType::STRING, buffer.as_str());

        Ok(true)
    }

    pub fn parse_word(&mut self) -> Result<bool, String> {
        let mut buffer: String = "".to_owned();
        let mut current = self.peek(None);
        loop {
            if !is_variable(current) {
                break
            }
            buffer.push(current);
            current = self.next();
        }

        if KEYWORDS.contains_key(buffer.as_str()) {
            self.add_token(KEYWORDS.get(buffer.as_str()).unwrap().to_owned(), buffer.as_str());
            return Ok(true)
        }

        self.add_token(TokenType::WORD, buffer.as_str());
        return Ok(true)
    }

    pub fn parse_comment(&mut self, multiline: Option<bool>) -> Result<bool, String> {
        if multiline.unwrap() {
            loop {
                let current = self.peek(None);
                if current.to_string() + &self.peek(Some(1)).to_string() == "*/" {
                    break
                }
                if current == '\0' {
                    return Err("Multiline comment did not close".to_string())
                }
                self.next();
            }
            self.next();
            self.next();
        }

        while "\r\n\0".to_string().contains(self.peek(None)) {
            self.next();
        }

        Ok(true)
    }

    pub fn peek(&self, pos: Option<usize>) -> char {
        let current = self.pos + pos.unwrap_or(0);
        if current >= self.code.len() {
            return '\0'
        }

        self.code.chars().nth(current).unwrap()
    }

    pub fn next(&mut self) -> char {
        self.pos += 1;
        self.peek(None)
    }

    pub fn add_token(&mut self, token_type: TokenType, text: &str) {
        let token = Token { token_type, text: text.to_owned() };
        self.tokens.push(token)
    }
}