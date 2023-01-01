use crate::lexer::{Token, TokenType};
use lazy_static::lazy_static;

lazy_static! {
    static ref EOF: Token = Token { token_type: TokenType::EOF, text: '\0'.to_string() };
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LogicalOp {
    OR,
    AND,
    EQ,
    NOTEQ,
    GTEQ,
    GT,
    LT,
    LTEQ,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum BinaryOp {
    PLUS,
    MINUS,
    MULTIPLY,
    DIVIDE,
    REMAINDER,
    EXPONENT
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum UnaryOp {
    MINUS,
    NOT
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum SwitchCase {
    Case(Node, Option<Node>),
    Default(Node),
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Node {
    Assign(Box<Node>, Box<Node>),

    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<Box<Node>>),

    Null(),

    // ArrayFun()

    Var(String),
    FieldAccess(Box<Node>, Vec<Box<Node>>),

    BlockStatement(Vec<Box<Node>>),
    IfElseStatement(Box<Node>, Box<Node>, Box<Option<Node>>),
    SwitchStatement(Box<Node>, Vec<SwitchCase>),
    // FIXME: args
    FunCall(Box<Node>, Vec<Box<Node>>),
    Return(Box<Node>),
    Fun(Box<Node>, Vec<String>, Box<Node>),
    Logical(LogicalOp, Box<Node>, Box<Node>),
    Binary(BinaryOp, Box<Node>, Box<Node>),
    Unary(UnaryOp, Box<Node>),
    Ternary(Box<Node>, Box<Node>)
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            pos: 0
        }
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        let mut root: Vec<Box<Node>> = vec![];

        while let Err(_b) = self.match_token(TokenType::EOF) {
            root.push(Box::new(self.statement()?))
        }

        Ok(Node::BlockStatement(root))
    }

    pub fn block(&mut self) -> Result<Node, String> {
        let mut root: Vec<Box<Node>> = vec![];

        self.match_token(TokenType::LBRACE);
        while let Err(_b) = self.match_token(TokenType::RBRACE) {
            root.push(Box::new(self.statement()?))
        }

        Ok(Node::BlockStatement(root))
    }

    pub fn statement_or_block(&mut self) -> Result<Node, String> {
        if let Ok(_b) = self.match_token(TokenType::LBRACE) {
            return Ok(self.block()?);
        }

        Ok(self.statement()?)
    }

    pub fn statement(&mut self) -> Result<Node, String> {
        let current = self.get_token(None);

        match current.token_type {
            TokenType::LET => {
                self.match_token(TokenType::LET);
                let name = self.consume_token(TokenType::WORD);
                self.consume_token(TokenType::EQUALS);
                let value = self.expression();

                println!("{:#?}", value);

                return Ok(
                    Node::Assign(
                        Box::new(
                            Node::Var(name?.text)
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
                let mut args: Vec<String> = vec![];
                while let Err(_b) = self.match_token(TokenType::RPAR) {
                    let arg = self.consume_token(TokenType::WORD);
                    args.push(arg?.text);
                    self.match_token(TokenType::COMMA);
                }
                let block = self.block();

                return Ok(
                    Node::Fun(
                        Box::new(
                            Node::Var(name?.text)
                        ), 
                        args,
                        Box::new(
                            block?
                        ),
                    )
                )
            },
            TokenType::IF => {
                self.match_token(TokenType::IF);
                self.consume_token(TokenType::LPAR);
                let condition = self.expression();
                self.consume_token(TokenType::RPAR);
                let if_statement = self.statement_or_block();

                let mut else_statement: Option<Node> = None;
                if let Ok(_b) = self.match_token(TokenType::ELSE) {
                    else_statement = Some(self.statement_or_block()?);
                }

                return Ok(
                    Node::IfElseStatement(
                        Box::new(condition?),
                        Box::new(if_statement?),
                        Box::new(else_statement)
                    )
                )
            },
            TokenType::SWITCH => {
                return Ok(self.switch_statement()?)
            },
            TokenType::RETURN => {
                self.match_token(TokenType::RETURN);
                let returning = self.expression();
                return Ok(Node::Return(Box::new(returning?)))
            }
            _ => {

            }
        }

        println!("{:#?}", current);

        panic!("Unknown statement")
    }

    pub fn switch_statement(&mut self) -> Result<Node, String> {
        self.match_token(TokenType::SWITCH);
        self.consume_token(TokenType::LPAR);
        // FIXME: variables only
        let variable = self.variable_expression();
        self.consume_token(TokenType::RPAR);

        let mut cases: Vec<SwitchCase> = vec![]; 

        self.match_token(TokenType::LBRACE);
        while let Err(_b) = self.match_token(TokenType::RBRACE) {
            let current = self.get_token(None);
            println!("{:#?}", current);
            match current.token_type {
                
                TokenType::DEFAULT => {
                    self.match_token(TokenType::DEFAULT);
                    let count_default_cases = cases.iter().filter(|&case| -> bool {
                        match case {
                            SwitchCase::Default(_) => true,
                            _ => false
                        }
                    }).count();

                    if count_default_cases == 1 {
                        return Err("Switch case can not have two or more default cases".to_string())
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
                        println!("{:#?}", statement)
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

    pub fn expression(&mut self) -> Result<Node, String> {
        Ok(self.ternary_expression()?)
    }

    pub fn primary_expression(&mut self) -> Result<Node, String> {
        let current = self.get_token(None);

        // FIXME
        match current.token_type {
            TokenType::WORD => Ok(self.variable_expression()?),

            TokenType::STRING |
            TokenType::NUMBER |
            TokenType::BOOLEAN |
            TokenType::LBRACKET |
            TokenType::NULL => Ok(self.value_expression()?),
            
            TokenType::SWITCH => Ok(self.switch_statement()?),
            _ => {
                println!("{:#?}", current);
                panic!("Unknown expression")
            }
        }
    }

    pub fn variable_suffixes(&mut self) -> Result<Vec<Box<Node>>, String>{
        let current = self.get_token(None);
        if current.token_type != TokenType::DOT && current.token_type != TokenType::LBRACKET {
            return Ok(vec![])
        }

        let mut indices = vec![];

        while current.token_type == TokenType::DOT || current.token_type == TokenType::LBRACKET {
            if self.match_token(TokenType::DOT).is_ok() {
                let field = self.consume_token(TokenType::WORD)?.text;
                indices.push(Box::new(Node::String(field)));
            }
            if self.match_token(TokenType::LBRACKET).is_ok() {
                indices.push(Box::new(self.expression()?));
                self.match_token(TokenType::RBRACKET);
            }
        }

        Ok(indices)
    }

    pub fn variable_fields_expression(&mut self, variable: Node) -> Result<Node, String> {
        let indices = self.variable_suffixes()?;

        if indices.len() > 0 {
            return Ok(Node::FieldAccess(Box::new(variable), indices))
        }

        Ok(variable)
    }

    pub fn variable_expression(&mut self) -> Result<Node, String> {
        let current = self.get_token(None);

        match current.token_type {
            TokenType::WORD => {
                self.match_token(current.token_type);
                let name = current.text;
                return Ok(Node::Var(name))
            }
            _ => {
                // FIXME: ?
                panic!("Unknown variable")
            }
        }
    }

    pub fn value_expression(&mut self) -> Result<Node, String> {
        let current = self.get_token(None);

        match current.token_type {
            TokenType::STRING => {
                self.match_token(current.token_type);
                let value = current.text;
                return Ok(Node::String(value))
            },
            TokenType::NUMBER => {
                self.match_token(current.token_type);
                let value = current.text.parse::<f64>().unwrap();
                return Ok(Node::Number(value))
            },
            TokenType::BOOLEAN => {
                self.match_token(current.token_type);
                return Ok(Node::Bool(current.text == "true"))
            },
            TokenType::NULL => {
                self.match_token(current.token_type);
                return Ok(Node::Null())
            },
            TokenType::LBRACKET => {
                self.match_token(TokenType::LBRACKET);
                let mut values = vec![];
                while let Err(_b) = self.match_token(TokenType::RBRACKET) {
                    values.push(Box::new(self.expression()?));
                    self.match_token(TokenType::COMMA);   
                }

                return Ok(Node::Array(values));
            }
            _ => {
                // FIXME: ?
                panic!("Unknown value")
            }
        }
    }

    pub fn ternary_expression(&mut self) -> Result<Node, String> {
        let mut result = self.logical_expression().unwrap();

        if self.match_token(TokenType::QUESTION).is_ok() {
            let true_condition = self.expression().unwrap();
            self.consume_token(TokenType::COLON);
            let false_condition = self.expression().unwrap();
            result = Node::Ternary(Box::new(true_condition), Box::new(false_condition));
        }

        Ok(result)
    }

    pub fn logical_expression(&mut self) -> Result<Node, String> {
        let mut result = self.unary_expression().unwrap();

        loop {
            if self.match_token(TokenType::BARBAR).is_ok() {
                result = Node::Logical(LogicalOp::OR, Box::new(result), Box::new(self.unary_expression().unwrap()));
            }
            if self.match_token(TokenType::AMPAMP).is_ok() {
                result = Node::Logical(LogicalOp::AND, Box::new(result), Box::new(self.unary_expression().unwrap()));
            }
            if self.match_token(TokenType::EQEQ).is_ok() {
                result = Node::Logical(LogicalOp::EQ, Box::new(result), Box::new(self.unary_expression().unwrap()));
            }
            if self.match_token(TokenType::EXCLEQ).is_ok() {
                result = Node::Logical(LogicalOp::NOTEQ, Box::new(result), Box::new(self.unary_expression().unwrap()));
            }
            if self.match_token(TokenType::GT).is_ok() {
                result = Node::Logical(LogicalOp::GT, Box::new(result), Box::new(self.unary_expression().unwrap()));
            }
            if self.match_token(TokenType::GTEQ).is_ok() {
                result = Node::Logical(LogicalOp::GTEQ, Box::new(result), Box::new(self.unary_expression().unwrap()));
            }
            if self.match_token(TokenType::LT).is_ok() {
                result = Node::Logical(LogicalOp::LT, Box::new(result), Box::new(self.unary_expression().unwrap()));
            }
            if self.match_token(TokenType::LTEQ).is_ok() {
                result = Node::Logical(LogicalOp::LTEQ, Box::new(result), Box::new(self.unary_expression().unwrap()));
            }
            break
        }

        Ok(result)
    }

    pub fn unary_expression(&mut self) -> Result<Node, String> {
        let mut result = self.binary_expression().unwrap();

        loop {
            if self.match_token(TokenType::PLUS).is_ok() {
                result = Node::Binary(BinaryOp::PLUS, Box::new(result), Box::new(self.binary_expression().unwrap()));
            }
            if self.match_token(TokenType::MINUS).is_ok() {
                result = Node::Binary(BinaryOp::MINUS, Box::new(result), Box::new(self.binary_expression().unwrap()));
            }
            if self.match_token(TokenType::STAR).is_ok() {
                result = Node::Binary(BinaryOp::MULTIPLY, Box::new(result), Box::new(self.binary_expression().unwrap()));
            }
            if self.match_token(TokenType::SLASH).is_ok() {
                result = Node::Binary(BinaryOp::DIVIDE, Box::new(result), Box::new(self.binary_expression().unwrap()));
            }
            if self.match_token(TokenType::PERCENT).is_ok() {
                result = Node::Binary(BinaryOp::REMAINDER, Box::new(result), Box::new(self.binary_expression().unwrap()));
            }
            if self.match_token(TokenType::DOUBLESTAR).is_ok() {
                result = Node::Binary(BinaryOp::EXPONENT, Box::new(result), Box::new(self.binary_expression().unwrap()));
            }
            break
        }

        Ok(result)
    }

    pub fn binary_expression(&mut self) -> Result<Node, String> {
        if self.match_token(TokenType::MINUS).is_ok() {
            return Ok(Node::Unary(UnaryOp::MINUS, Box::new(self.primary_expression().unwrap())));
        }
        // FIXME: multiple not
        if self.match_token(TokenType::EXCL).is_ok() {
            return Ok(Node::Unary(UnaryOp::NOT, Box::new(self.primary_expression().unwrap())));
        }

        Ok(self.primary_expression().unwrap())
    }

    pub fn consume_token(&mut self, token_type: TokenType) -> Result<Token, String> {
        let current = self.get_token(None);
        if current.token_type != token_type {
            panic!("Token {:#?} didnt match {:#?}", token_type, current.token_type);
        }

        self.pos += 1;
        Ok(current)
    }

    pub fn match_token(&mut self, token_type: TokenType) -> Result<bool, bool> {
        let current = self.get_token(None);
        if current.token_type != token_type {
            return Err(false)
        }
        self.pos += 1;
        Ok(true)
    }

    pub fn get_token(&self, pos: Option<usize>) -> Token {
        let current = self.pos + pos.unwrap_or(0);
        if current >= self.tokens.len() {
            return EOF.to_owned()
        }

        self.tokens.iter().peekable().nth(current).unwrap().to_owned()
    }
}