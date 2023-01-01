use core::panic;

use crate::parser::{ Node, SwitchCase, LogicalOp, BinaryOp, UnaryOp };

pub mod scope;
pub mod types;

use self::{scope::Scope, types::CocoValue};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&mut self, node: Node, scope: &mut Scope) -> Result<CocoValue, String>  {
        self.walk_tree(node, scope)
    }

    pub fn walk_tree(&mut self, node: Node, scope: &mut Scope) -> Result<CocoValue, String> {
        match node {
            Node::BlockStatement(statements) => {
                let mut result = CocoValue::CocoNull;
                for statement in statements {
                    match *statement {
                        Node::Return(value) => {
                            result = self.walk_tree(*value, scope)?;
                            break;
                        },
                        _ => {
                            self.walk_tree(*statement, scope)?;
                        }
                    }
                }
                Ok(result)
            },
            Node::Assign(variable, value) => {
                match *variable {
                    Node::Var(name) => {
                        let value = self.walk_tree(*value, scope)?;
                        
                        return Ok(scope.set(name, value))
                    },
                    _ => {
                        panic!("Unexpected assign")
                    }
                }
            },
            Node::Var(name) => Ok(scope.get(name).to_owned()),
            /*Node::FieldAccess(variable, indices) => {
                
            },*/
            Node::String(value) => Ok(CocoValue::CocoString(value)),
            Node::Number(value) => Ok(CocoValue::CocoNumber(value)),
            Node::Bool(value) => Ok(CocoValue::CocoBoolean(value)),
            Node::Array(value) => {
                let mut array_values = vec![];
                for node in value {
                    let value = self.walk_tree(*node, scope)?;
                    array_values.push(Box::new(value))
                }

                Ok(CocoValue::CocoArray(array_values))
            },
            Node::Logical(operator, node1, node2) => {
                let val1 = self.walk_tree(*node1, scope);
                let val2 = self.walk_tree(*node2, scope);
                
                match operator {
                    LogicalOp::AND => Ok(CocoValue::CocoBoolean(val1?.as_bool() && val2?.as_bool())),
                    LogicalOp::OR => Ok(CocoValue::CocoBoolean(val1?.as_bool() && val2?.as_bool())),
                    LogicalOp::EQ => Ok(CocoValue::CocoBoolean(val1? == val2?)),
                    LogicalOp::NOTEQ => Ok(CocoValue::CocoBoolean(val1? != val2?)),
                    LogicalOp::GT => Ok(CocoValue::CocoBoolean(val1? > val2?)),
                    LogicalOp::GTEQ => Ok(CocoValue::CocoBoolean(val1? >= val2?)),
                    LogicalOp::LT => Ok(CocoValue::CocoBoolean(val1? < val2?)),
                    LogicalOp::LTEQ => Ok(CocoValue::CocoBoolean(val1? <= val2?))
                }
            },
            Node::Binary(operator, node1, node2) => {
                let val1 = self.walk_tree(*node1, scope)?;
                let val2 = self.walk_tree(*node2, scope)?;
                
                match operator {
                    BinaryOp::PLUS => {
                        match val1.clone() {
                            CocoValue::CocoString(val) => Ok(CocoValue::CocoString(val.to_string() + &val2.as_string())),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val + &val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoString(val1.clone().as_string() + &val2.as_string())),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() + &val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoString(val1.clone().as_string() + &val2.as_string())),
                            CocoValue::CocoNull => Ok(val2)
                        }
                    },
                    BinaryOp::MINUS => {
                        match val1.clone() {
                            CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val - &val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.clone().as_number() - &val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(-&val2.as_number()))
                        }
                    },
                    BinaryOp::MULTIPLY => {
                        match val1.clone() {
                            CocoValue::CocoString(val) => Ok(CocoValue::CocoString(val.to_string().repeat(val2.as_number() as usize))),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val * &val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() * &val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0))
                        }
                    },
                    BinaryOp::DIVIDE => {
                        match val1.clone() {
                            CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(val1.as_number() / &val2.as_number())),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val / &val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() / &val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0))
                        }
                    },
                    BinaryOp::REMAINDER => {
                        match val1.clone() {
                            CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(val1.as_number() % &val2.as_number())),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val % &val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() % &val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0))
                        }
                    },
                    BinaryOp::EXPONENT => {
                        match val1.clone() {
                            CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(val1.as_number().powf(val2.as_number()))),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val.powf(val2.as_number()))),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number().powf(val2.as_number()))),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0))
                        }
                    }
                }
            },
            Node::Unary(operator, node) => {
                let value = self.walk_tree(*node, scope)?;

                match operator {
                    UnaryOp::MINUS => {
                        match value.clone() {
                            CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(-value.as_number())),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(-val)),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(-value.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(-0.0))
                        }
                    },
                    UnaryOp::NOT => {
                        Ok(CocoValue::CocoBoolean(!value.as_bool()))
                    }
                }
            },
            Node::Fun(variable, args, block) => {
                match *variable {
                    Node::Var(name) => Ok(scope.set(name, CocoValue::CocoFunction(args, *block))),
                    _ => {
                        panic!("Unexpected assign")
                    }
                }
            },
            Node::SwitchStatement(variable, switch_cases) => {
                let value = self.walk_tree(*variable, scope);

                let mut iter = switch_cases.iter();

                loop {
                    let case = iter.next();
                    match case.unwrap() {
                        SwitchCase::Case(val, statement) => {
                            if statement.is_none() {
                                loop {
                                    let next_case = iter.next();
                                    match next_case.unwrap() {
                                        SwitchCase::Default(next_default_statement) => {
                                            let next_default_statement_value = self.walk_tree(next_default_statement.to_owned(), scope);

                                            println!("{:#?}", next_default_statement);

                                            return Ok(next_default_statement_value?);
                                        },
                                        SwitchCase::Case(next_val, next_statement) => {
                                            if next_statement.is_none() {
                                                continue;
                                            }

                                            let next_val_value = self.walk_tree(next_val.to_owned(), scope);
                                            let next_statement_value = self.walk_tree(next_statement.to_owned().unwrap(), scope);

                                            if next_val_value == value {
                                                return Ok(next_statement_value?)
                                            }

                                            continue;
                                        }
                                    }
                                } 
                            }

                            let node_val = self.walk_tree(val.to_owned(), scope);
                            let statement_value = self.walk_tree(statement.to_owned().unwrap(), scope);
                            if node_val == value {
                                return Ok(statement_value?)
                            }

                            continue;
                        },
                        SwitchCase::Default(statement) => {
                            let statement_value = self.walk_tree(statement.to_owned(), scope);

                            return Ok(statement_value?)
                        }
                    }
                }
            }
            _ => Ok(CocoValue::CocoNull)
        }
    }
}