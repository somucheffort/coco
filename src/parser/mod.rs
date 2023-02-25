use std::collections::{ BTreeMap };

use crate::{lexer::{ Token, TokenType }, interpreter::types::{FunctionArguments, FunctionArgument}, Error, Resolver};
use phf::phf_map;

const ASSIGNOP: phf::Map<&str, AssignmentOp> = phf_map! {
    "=" => AssignmentOp::EQ,
    "+=" =>  AssignmentOp::PLUSEQ,
    "-=" =>  AssignmentOp::MINUSEQ,
    "*=" =>  AssignmentOp::MULEQ,
    "/=" => AssignmentOp::DIVEQ,
    "%=" =>  AssignmentOp::REMEQ,
    "**=" =>  AssignmentOp::EXPEQ,
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum AssignmentOp {
    EQ,      // a = 1
    PLUSEQ,  // a += 1
    MINUSEQ, // a -= 1
    MULEQ,   // a *= 1
    DIVEQ,   // a /= 1
    REMEQ,   // a %= 1
    EXPEQ,   // a **= 1
    // MINUSMINUS
    // PLUSPLUS
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LogicalOp {
    OR,    // ||
    AND,   // &&
    EQ,    // ==
    NOTEQ, // !=
    GTEQ,  // >=
    GT,    // >
    LT,    // <
    LTEQ,  // <=
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum BinaryOp {
    PLUS,      // +
    MINUS,     // -
    MULTIPLY,  // *
    DIVIDE,    // /
    REMAINDER, // %
    EXPONENT   // **
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum UnaryOp {
    MINUS, // -a
    NOT    // !a
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum SwitchCase {
    Case(Node, Option<Node>),
    Default(Node),
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Node {
    Import(String),
    ImportFrom(String, Vec<String>),

    Assign(Box<Node>, Box<Node>),
    AssignOp(AssignmentOp, Box<Node>, Box<Node>),

    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<Box<Node>>),
    Object(BTreeMap<String, Box<Node>>),
    Class(String, Option<Box<Node>>, BTreeMap<String, Node>),
    Null,

    // ArrayFun()

    Var(String),
    FieldAccess(Box<Node>, Vec<Box<Node>>),

    Range(Box<Node>, Box<Node>, bool),

    BlockStatement(Vec<Box<Node>>),
    IfElseStatement(Box<Node>, Box<Node>, Box<Option<Node>>),
    WhileStatement(Box<Node>, Box<Node>),
    ForStatement(String, Box<Node>, Box<Node>),
    SwitchStatement(Box<Node>, Vec<SwitchCase>),
    // FIXME: args
    FunCall(Box<Node>, Vec<Box<Node>>),
    Return(Box<Node>),
    Fun(Box<Node>, FunctionArguments, Box<Node>),
    Logical(LogicalOp, Box<Node>, Box<Node>),
    Binary(BinaryOp, Box<Node>, Box<Node>),
    Unary(UnaryOp, Box<Node>),
    Ternary(Box<Node>, Box<Node>, Box<Node>)
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    resolver: Resolver
}

impl Parser {
    pub fn new(tokens: Vec<Token>, resolver: &Resolver) -> Self {
        Self {
            tokens,
            pos: 0,
            resolver: resolver.to_owned()
        }
    }

    pub fn parse(&mut self) -> Result<Node, Error> {
        let mut root: Vec<Box<Node>> = vec![];

        while !self.match_token(TokenType::EOF) {
            root.push(Box::new(self.statement()?))
        }

        Ok(Node::BlockStatement(root))
    }

    pub fn block(&mut self) -> Result<Node, Error> {
        let mut root: Vec<Box<Node>> = vec![];

        self.match_token(TokenType::LBRACE);
        while !self.match_token(TokenType::RBRACE) {
            root.push(Box::new(self.statement()?))
        }

        Ok(Node::BlockStatement(root))
    }

    pub fn statement_or_block(&mut self) -> Result<Node, Error> {
        if self.match_token(TokenType::LBRACE) {
            return self.block()
        }

        self.statement()
    }

    pub fn statement(&mut self) -> Result<Node, Error> {
        let current = self.get_token(None);

        match current.token_type {
            TokenType::LET => {
                self.match_token(TokenType::LET);
                let name = self.consume_token(TokenType::WORD);
                self.consume_token(TokenType::EQUALS);
                let value = self.expression();

                Ok(
                    Node::Assign(
                        Box::new(
                            Node::Var(name.text)
                        ), 
                        Box::new(
                            value?
                        ),
                    )
                )
            },
            TokenType::FUN => {
                self.match_token(TokenType::FUN);
                let name = self.consume_token(TokenType::WORD);
                self.consume_token(TokenType::LPAR);
                let mut args: FunctionArguments = FunctionArguments::new(vec![]);
                while !self.match_token(TokenType::RPAR) {
                    let arg = self.consume_token(TokenType::WORD);
                    args.add(FunctionArgument::Required(arg.text));
                    self.match_token(TokenType::COMMA);
                }
                let block = self.block();

                Ok(
                    Node::Fun(
                        Box::new(
                            Node::Var(name.text)
                        ), 
                        args,
                        Box::new(
                            block?
                        ),
                    )
                )
            },
            TokenType::CLASS => {
                self.match_token(TokenType::CLASS);
                let class_name = self.consume_token(TokenType::WORD).text;
                // TODO extending
                self.match_token(TokenType::LBRACE);
                let mut prototype: BTreeMap<String, Node> = BTreeMap::default();
                let mut constructor = None;
                while !self.match_token(TokenType::RBRACE) {
                    let class_current = self.get_token(None);

                    if class_current.token_type == TokenType::WORD {
                        let name = self.consume_token(TokenType::WORD).text;
                        // TODO vars
                        self.consume_token(TokenType::LPAR);
                        let mut args: FunctionArguments = FunctionArguments::new(vec![]);
                        while !self.match_token(TokenType::RPAR) {
                            let arg = self.consume_token(TokenType::WORD);
                            args.add(FunctionArgument::Required(arg.text));
                            self.match_token(TokenType::COMMA);
                        }
                        let block = self.block();

                        if name == "constructor" {
                            constructor = Some(Box::new(Node::Fun(
                                Box::new(
                                    Node::Var(name)
                                ), 
                                args,
                                Box::new(
                                    block?
                                ),
                            )));
                        } else {
                            prototype.insert(name.clone(), Node::Fun(
                                Box::new(
                                    Node::Var(name)
                                ), 
                                args,
                                Box::new(
                                    block?
                                ),
                            ));
                        }
                    }
                }

                Ok(Node::Class(class_name, constructor, prototype))
            }
            TokenType::IF => {
                self.match_token(TokenType::IF);
                self.consume_token(TokenType::LPAR);
                let condition = self.expression();
                self.consume_token(TokenType::RPAR);
                let if_statement = self.statement_or_block();

                let mut else_statement: Option<Node> = None;
                if self.match_token(TokenType::ELSE) {
                    else_statement = Some(self.statement_or_block()?);
                }

                Ok(
                    Node::IfElseStatement(
                        Box::new(condition?),
                        Box::new(if_statement?),
                        Box::new(else_statement)
                    )
                )
            },
            TokenType::FOR => {
                self.match_token(TokenType::FOR);
                self.consume_token(TokenType::LPAR);
                let variable = self.consume_token(TokenType::WORD).text;
                self.consume_token(TokenType::IN);
                let iterator = self.expression()?;
                self.consume_token(TokenType::RPAR);
                let block = self.block()?;

                Ok(
                    Node::ForStatement(
                        variable,
                        Box::new(iterator),
                        Box::new(block)
                    )
                )
            },
            TokenType::WHILE => {
                self.match_token(TokenType::WHILE);
                self.consume_token(TokenType::LPAR);
                let condition = self.expression()?;
                self.consume_token(TokenType::RPAR);
                let block = self.block()?;

                Ok(Node::WhileStatement(Box::new(condition), Box::new(block)))
            },
            TokenType::SWITCH => self.switch_statement(),
            TokenType::RETURN => {
                self.match_token(TokenType::RETURN);
                let returning = self.expression();
                Ok(Node::Return(Box::new(returning?)))
            },
            TokenType::IMPORT => {
                // FIXME
                self.match_token(TokenType::IMPORT);
                if self.get_token(None).token_type == TokenType::STRING {
                    let lib_name = self.consume_token(TokenType::STRING).text;

                    return Ok(Node::Import(lib_name))
                }
                let mut libs = vec![];
                while self.get_token(None).token_type == TokenType::WORD {
                    libs.push(self.consume_token(TokenType::WORD).text);
                    // if let Err(_b) = self.match_token(TokenType::COMMA) {
                    if !self.match_token(TokenType::COMMA) {
                        break
                    }
                }

                self.consume_token(TokenType::FROM);
                let lib_name = self.consume_token(TokenType::STRING).text;
                Ok(Node::ImportFrom(lib_name, libs))
            },
            _ => Ok(self.expression()?)
        }
    }

    pub fn switch_statement(&mut self) -> Result<Node, Error> {
        self.match_token(TokenType::SWITCH);
        self.consume_token(TokenType::LPAR);
        // FIXME: variables only
        let variable = self.variable_expression();
        self.consume_token(TokenType::RPAR);

        let mut cases: Vec<SwitchCase> = vec![]; 

        self.match_token(TokenType::LBRACE);
        while !self.match_token(TokenType::RBRACE) {
            let current = self.get_token(None);
            match current.token_type {
                
                TokenType::DEFAULT => {
                    self.match_token(TokenType::DEFAULT);
                    let count_default_cases = cases.iter().filter(|&case| -> bool {
                        matches!(case, SwitchCase::Default(_))
                    }).count();

                    if count_default_cases == 1 {
                        return Err(Error {
                            msg: "Switch case can not have two or more default cases".to_string(),
                            pos: self.resolver.resolve_where(self.get_token(None).pos)
                        })
                    }

                    self.consume_token(TokenType::COLON);
                    cases.push(SwitchCase::Default(self.statement_or_block()?))
                },
                TokenType::CASE => {
                    self.match_token(TokenType::CASE);
                    // FIXME: values only
                    let value = self.value_expression();
                    self.consume_token(TokenType::COLON);
                    let case_current = self.get_token(None);

                    let mut statement = None;

                    if case_current.token_type != TokenType::CASE && case_current.token_type != TokenType::DEFAULT {
                        statement = Some(self.statement_or_block()?);
                    }
                    cases.push(SwitchCase::Case(value?, statement))
                },
                _ => {}
            }
        }

        Ok(
            Node::SwitchStatement(
                Box::new(variable?), 
                cases
            )
        )
    }

    pub fn expression(&mut self) -> Result<Node, Error> {
        let assign = self.assignment_expression().unwrap();

        if let Some(a) = assign {
            return Ok(a)
        }

        self.ternary_expression()
    }

    pub fn primary_expression(&mut self) -> Result<Node, Error> {
        let current = self.get_token(None);

        // FIXME
        match current.token_type {
            TokenType::WORD |

            TokenType::STRING |
            TokenType::NUMBER |
            TokenType::BOOLEAN |
            TokenType::LBRACKET |
            TokenType::LBRACE |
            TokenType::NULL |
            TokenType::NAN => {
                let var_val = self.var_val_expression()?;
                let field_access = self.field_access_expression(var_val)?;

                if self.get_token(None).token_type == TokenType::LPAR {
                    return self.function_chain_expression(field_access)
                }

                Ok(field_access)
            },

            TokenType::LPAR => {
                self.match_token(TokenType::LPAR);
                let expr = self.expression()?;
                self.match_token(TokenType::RPAR);
                Ok(expr)
            },
            
            TokenType::SWITCH => Ok(self.switch_statement()?),

            TokenType::NEW => {
                self.match_token(TokenType::NEW);
                let var = self.variable_expression()?;
                let field_access = self.field_access_expression(var)?;

                self.function_chain_expression(field_access)
            }

            _ => {
                //println!("{:#?}", current);
                Err(Error {
                    msg: "Unknown expression".to_string(),
                    pos: self.resolver.resolve_where(self.get_token(None).pos)
                })
            }
        }
    }

    pub fn function_chain_expression(&mut self, variable: Node) -> Result<Node, Error> {
        let fun_call = self.function_call_expression(variable);

        if self.get_token(None).token_type == TokenType::LPAR {
            return self.function_chain_expression(fun_call?);
        }

        if self.get_token(None).token_type == TokenType::DOT {
            let suffixes = self.variable_suffixes().unwrap();
            if suffixes.is_empty() {
                return fun_call;
            }

            if self.get_token(None).token_type == TokenType::LPAR {
                return self.function_chain_expression(Node::FieldAccess(Box::new(fun_call?), suffixes));
            }

            return Ok(Node::FieldAccess(Box::new(fun_call?), suffixes))
        }

        fun_call
    }

    pub fn function_call_expression(&mut self, variable: Node) -> Result<Node, Error> {
        self.consume_token(TokenType::LPAR);
        let mut args = vec![];

        while !self.match_token(TokenType::RPAR) {
            args.push(Box::new(self.expression()?));
            self.match_token(TokenType::COMMA);
        }

        Ok(Node::FunCall(Box::new(variable), args))
    }

    pub fn var_val_expression(&mut self) -> Result<Node, Error> {
        if self.get_token(None).token_type == TokenType::WORD {
            return self.variable_expression()
        }

        self.value_expression()
    }

    pub fn variable_suffixes(&mut self) -> Result<Vec<Box<Node>>, String>{
        let current = self.get_token(None);
        if current.token_type != TokenType::DOT && current.token_type != TokenType::LBRACKET {
            return Ok(vec![])
        }

        let mut indices = vec![];

        while self.get_token(None).token_type == TokenType::DOT || self.get_token(None).token_type == TokenType::LBRACKET {
            if self.match_token(TokenType::DOT) {
                let field = self.consume_token(TokenType::WORD).text;
                indices.push(Box::new(Node::String(field)));
            }
            if self.match_token(TokenType::LBRACKET) {
                indices.push(Box::new(self.expression().unwrap()));
                self.match_token(TokenType::RBRACKET);
            }
        } 

        Ok(indices)
    }

    pub fn field_access_expression(&mut self, variable: Node) -> Result<Node, Error> {
        let indices = self.variable_suffixes().unwrap();

        if !indices.is_empty() {
            return Ok(Node::FieldAccess(Box::new(variable), indices))
        }

        Ok(variable)
    }

    pub fn variable_expression(&mut self) -> Result<Node, Error> {
        let current = self.get_token(None);

        match current.token_type {
            TokenType::WORD => {
                self.match_token(current.token_type);
                let name = current.text;
                Ok(Node::Var(name))
            }
            _ => {
                // FIXME: ?
                Err(Error {
                    msg: "Unknown variable".to_string(),
                    pos: self.resolver.resolve_where(self.get_token(None).pos)
                })
            }
        }
    }

    pub fn range_expression(&mut self, from: Node) -> Result<Node, Error> {
        let inclusive = self.match_token(TokenType::EQUALS);
        let to = self.var_val_expression()?;
        
        Ok(
            Node::Range(
                Box::new(from),
                Box::new(to),
                inclusive
            )
        )
    }

    pub fn value_expression(&mut self) -> Result<Node, Error> {
        let current = self.get_token(None);

        match current.token_type {
            TokenType::STRING => {
                self.match_token(current.token_type);
                let value = current.text;
                Ok(Node::String(value))
            },
            TokenType::NUMBER => {
                self.match_token(current.token_type);
                let value = current.text.parse::<f64>().unwrap();
                let node = Node::Number(value);

                if self.match_token(TokenType::DOTDOT) {
                    return self.range_expression(node)
                }
                
                Ok(node)
            },
            TokenType::BOOLEAN => {
                self.match_token(current.token_type);
                Ok(Node::Bool(current.text == "true"))
            },
            TokenType::NULL => {
                self.match_token(current.token_type);
                Ok(Node::Null)
            },
            TokenType::NAN => {
                self.match_token(current.token_type);
                Ok(Node::Number(f64::NAN))
            },
            TokenType::LBRACKET => {
                self.match_token(TokenType::LBRACKET);
                let mut values = vec![];
                while !self.match_token(TokenType::RBRACKET) {
                    values.push(Box::new(self.expression()?));
                    self.match_token(TokenType::COMMA);   
                }

                Ok(Node::Array(values))
            },
            TokenType::LBRACE => {
                self.match_token(TokenType::LBRACE);
                let mut map = BTreeMap::new();
                while !self.match_token(TokenType::RBRACE) {
                    let name = self.consume_token(TokenType::WORD).text;
                    self.consume_token(TokenType::COLON);
                    map.insert(name, Box::new(self.expression()?));
                    self.match_token(TokenType::COMMA);   
                }

                Ok(Node::Object(map))
            },
            _ => {
                // FIXME: ?
                panic!("Unknown value")
            }
        }
    }

    pub fn assignment_expression(&mut self) -> Result<Option<Node>, String> {
        let pre_pos = self.pos;
        let variable = self.variable_expression();
        if variable.is_err() {
            self.pos = pre_pos;
            return Ok(None);
        }
        let field_access = self.field_access_expression(variable.unwrap());

        let current = self.get_token(None);

        if !ASSIGNOP.contains_key(&current.text) {
            self.pos = pre_pos;
            return Ok(None);
        }
        self.match_token(current.token_type);

        let op = ASSIGNOP.get(&current.text).unwrap();

        Ok(Some(Node::AssignOp(op.to_owned(), Box::new(field_access.unwrap()), Box::new(self.expression().unwrap()))))
    } 

    pub fn ternary_expression(&mut self) -> Result<Node, Error> {
        let mut result = self.logical_or_expression()?;

        if self.match_token(TokenType::QUESTION) {
            let true_condition = self.expression()?;
            self.consume_token(TokenType::COLON);
            let false_condition = self.expression()?;
            result = Node::Ternary(Box::new(result), Box::new(true_condition), Box::new(false_condition));
        }

        Ok(result)
    }

    pub fn logical_or_expression(&mut self) -> Result<Node, Error> {
        let mut result = self.logical_and_expression()?;
        loop {
            if self.match_token(TokenType::BARBAR) {
                result = Node::Logical(LogicalOp::OR, Box::new(result), Box::new(self.logical_and_expression()?));
                continue;
            }
            break
        }

        Ok(result)
    }

    pub fn logical_and_expression(&mut self) -> Result<Node, Error> {
        let mut result = self.logical_eq_expression()?;
        loop {
            if self.match_token(TokenType::AMPAMP) {
                result = Node::Logical(LogicalOp::AND, Box::new(result), Box::new(self.logical_eq_expression()?));
                continue;
            }
            break
        }

        Ok(result)
    }

    pub fn logical_eq_expression(&mut self) -> Result<Node, Error> {
        let mut result = self.logical_cond_expression()?;
        loop {
            if self.match_token(TokenType::EQEQ) {
                result = Node::Logical(LogicalOp::EQ, Box::new(result), Box::new(self.logical_cond_expression()?));
                continue;
            }
            if self.match_token(TokenType::EXCLEQ) {
                result = Node::Logical(LogicalOp::NOTEQ, Box::new(result), Box::new(self.logical_cond_expression()?));
                continue;
            }
            break
        }

        Ok(result)
    }

    pub fn logical_cond_expression(&mut self) -> Result<Node, Error> {
        let mut result = self.binary_add_expression()?;
        loop {
            if self.match_token(TokenType::GT) {
                result = Node::Logical(LogicalOp::GT, Box::new(result), Box::new(self.binary_add_expression()?));
                continue;
            }
            if self.match_token(TokenType::GTEQ) {
                result = Node::Logical(LogicalOp::GTEQ, Box::new(result), Box::new(self.binary_add_expression()?));
                continue;
            }
            if self.match_token(TokenType::LT) {
                result = Node::Logical(LogicalOp::LT, Box::new(result), Box::new(self.binary_add_expression()?));
                continue;
            }
            if self.match_token(TokenType::LTEQ) {
                result = Node::Logical(LogicalOp::LTEQ, Box::new(result), Box::new(self.binary_add_expression()?));
                continue;
            }
            break
        }

        Ok(result)
    }

    pub fn binary_add_expression(&mut self) -> Result<Node, Error> {
        let mut result = self.binary_mul_expression()?;

        loop {
            if self.match_token(TokenType::PLUS) {
                result = Node::Binary(BinaryOp::PLUS, Box::new(result), Box::new(self.binary_mul_expression()?));
                continue;
            }
            if self.match_token(TokenType::MINUS) {
                result = Node::Binary(BinaryOp::MINUS, Box::new(result), Box::new(self.binary_mul_expression()?));
                continue;
            }
            break;
        }

        Ok(result)
    }

    pub fn binary_mul_expression(&mut self) -> Result<Node, Error>  {
        let mut result = self.unary_expression()?;
        loop {
            if self.match_token(TokenType::STAR) {
                result = Node::Binary(BinaryOp::MULTIPLY, Box::new(result), Box::new(self.unary_expression()?));
                continue;
            }
            if self.match_token(TokenType::SLASH) {
                result = Node::Binary(BinaryOp::DIVIDE, Box::new(result), Box::new(self.unary_expression()?));
                continue;
            }
            if self.match_token(TokenType::PERCENT) {
                result = Node::Binary(BinaryOp::REMAINDER, Box::new(result), Box::new(self.unary_expression()?));
                continue;
            } 
            if self.match_token(TokenType::DOUBLESTAR) {
                result = Node::Binary(BinaryOp::EXPONENT, Box::new(result), Box::new(self.unary_expression()?));
                continue;
            }
            break;
        }

        Ok(result)
    }

    pub fn unary_expression(&mut self) -> Result<Node, Error> {
        if self.match_token(TokenType::MINUS) {
            return Ok(Node::Unary(UnaryOp::MINUS, Box::new(self.expression()?)))
        } else if self.match_token(TokenType::EXCL) {
            return Ok(Node::Unary(UnaryOp::NOT, Box::new(self.expression()?)));
        }

        self.primary_expression()
    }

    pub fn consume_token(&mut self, token_type: TokenType) -> Token {
        let current = self.get_token(None);
        if current.token_type != token_type {
            self.resolver.exit_error(
                format!("Token {:#?} didnt match {:#?}", token_type, current.token_type),
                vec![0,0]
            );
        }

        self.pos += 1;
        current
    }

    pub fn match_token(&mut self, token_type: TokenType) -> bool {
        let current = self.get_token(None);
        if current.token_type != token_type {
            return false
        }

        self.pos += 1;
        true
    }

    pub fn get_token(&self, pos: Option<usize>) -> Token {
        let current = self.pos + pos.unwrap_or(0);
        if current >= self.tokens.len() {
            return Token { 
                token_type: TokenType::EOF, 
                text: "\0".to_string(), 
                pos: self.tokens.len() + 1
            }
        }

        self.tokens.iter().peekable().nth(current).unwrap().to_owned()
    }
}