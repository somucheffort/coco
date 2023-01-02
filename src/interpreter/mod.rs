use core::panic;
use std::{env::Args, collections::BTreeMap};

use crate::parser::{ Node, SwitchCase, LogicalOp, BinaryOp, UnaryOp };

pub mod scope;
pub mod types;

use self::{scope::Scope, types::{CocoValue, FieldAccessor, Fun}};

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
                        
                        Ok(scope.set(name, value))
                    },
                    _ => {
                        panic!("Unexpected assign")
                    }
                }
            },
            Node::Var(name) => Ok(scope.get(name).to_owned()),
            Node::FieldAccess(variable, indices) => {
                let value = self.walk_tree(*variable, scope)?;
                let fields = indices.iter().map(|i| self.walk_tree(*i.to_owned(), scope).unwrap_or(CocoValue::CocoNull)).collect::<Vec<CocoValue>>();
                let field_accessor = FieldAccessor::new(value, fields);
                Ok(field_accessor.get())
            },
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
            Node::Object(map) => Ok(
                CocoValue::CocoObject(
                    map
                    .into_iter()
                    .map(|x| (x.0, Box::new(self.walk_tree(*x.1, scope).unwrap())))
                    .collect::<BTreeMap<String, Box<CocoValue>>>()
                )
            ),
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
                            CocoValue::CocoString(val) => Ok(CocoValue::CocoString(val + &val2.as_string())),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val + val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoString(val1.as_string() + &val2.as_string())),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() + val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoString(val1.as_string() + &val2.as_string())),
                            CocoValue::CocoObject(_map) => Ok(CocoValue::CocoString(val1.as_string() + &val2.as_string())),
                            CocoValue::CocoNull => Ok(val2)
                        }
                    },
                    BinaryOp::MINUS => {
                        match val1.clone() {
                            CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val - val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() - val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(-&val2.as_number()))
                        }
                    },
                    BinaryOp::MULTIPLY => {
                        match val1.clone() {
                            CocoValue::CocoString(val) => Ok(CocoValue::CocoString(val.repeat(val2.as_number() as usize))),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val * val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() * val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0))
                        }
                    },
                    BinaryOp::DIVIDE => {
                        match val1.clone() {
                            CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(val1.as_number() / val2.as_number())),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val / val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() / val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0))
                        }
                    },
                    BinaryOp::REMAINDER => {
                        match val1.clone() {
                            CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(val1.as_number() % val2.as_number())),
                            CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val % val2.as_number())),
                            CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() % val2.as_number())),
                            CocoValue::CocoFunction(_a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                            CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
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
                            CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
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
                            CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
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
                    Node::Var(name) => Ok(scope.set(name, CocoValue::CocoFunction(args, Fun::Node(*block)))),
                    _ => {
                        panic!("Unexpected assign")
                    }
                }
            },
            Node::FunCall(variable, args) => {
                let value = self.walk_tree(*variable, scope)?;
                let args_val = args.iter()
                .map(|arg| self.walk_tree(*arg.to_owned(), scope).unwrap())
                .collect::<Vec<CocoValue>>();

                match value {
                    CocoValue::CocoFunction(fun_args, fun_block) => {
                        match fun_block {
                            Fun::Node(block) => {
                                let mut fun_scope = Scope::new(Some(Box::new(scope.to_owned())));

                                for (i, arg) in fun_args.iter().enumerate() {
                                    let current_arg = self.walk_tree(*args.get(i).unwrap().to_owned(), scope)?;
                                    fun_scope.set(arg.to_owned(), current_arg);
                                }

                                self.walk_tree(block, &mut fun_scope)
                            },
                            Fun::Builtin(f) => {
                                Ok(f(args_val))
                            }
                        }
                        
                    },
                    _ => panic!("FIXME")
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

                                            //println!("{:#?}", next_default_statement);

                                            return next_default_statement_value;
                                        },
                                        SwitchCase::Case(next_val, next_statement) => {
                                            if next_statement.is_none() {
                                                continue;
                                            }

                                            let next_val_value = self.walk_tree(next_val.to_owned(), scope);
                                            let next_statement_value = self.walk_tree(next_statement.to_owned().unwrap(), scope);

                                            if next_val_value == value {
                                                return next_statement_value
                                            }

                                            continue;
                                        }
                                    }
                                } 
                            }

                            let node_val = self.walk_tree(val.to_owned(), scope);
                            let statement_value = self.walk_tree(statement.to_owned().unwrap(), scope);
                            if node_val == value {
                                return statement_value
                            }

                            continue;
                        },
                        SwitchCase::Default(statement) => {
                            let statement_value = self.walk_tree(statement.to_owned(), scope);

                            return statement_value
                        }
                    }
                }
            }
            _ => Ok(CocoValue::CocoNull)
        }
    }
}