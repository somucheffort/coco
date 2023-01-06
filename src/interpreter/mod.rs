use core::panic;
use std::{collections::{BTreeMap, HashMap}, cmp::Ordering};

use crate::{parser::{ Node, SwitchCase, LogicalOp, BinaryOp, UnaryOp, AssignmentOp }, modules::import_module};

pub mod scope;
pub mod types;

use self::{scope::Scope, types::{CocoValue, FieldAccessor, FuncImpl, create_string, FuncArg}};

pub struct Interpreter {}

pub fn walk_tree(node: Node, scope: &mut Scope) -> Result<CocoValue, String> {
    match node {
        Node::Import(module) => {
            import_module(module.as_str(), scope, None);
            Ok(CocoValue::CocoNull)
        },
        Node::ImportFrom(module, objects) => {
            import_module(module.as_str(), scope, Some(objects));
            Ok(CocoValue::CocoNull)
        },
        Node::BlockStatement(statements) => {
            let mut result = CocoValue::CocoNull;
            for statement in statements {
                match *statement {
                    Node::Return(value) => {
                        result = walk_tree(*value, scope)?;
                        break;
                    },
                    _ => {
                        walk_tree(*statement, scope)?;
                    }
                }
            }
            Ok(result)
        },
        Node::Assign(variable, value) => {
            match *variable {
                Node::Var(name) => {
                    let value = walk_tree(*value, scope)?;
                    
                    Ok(scope.set(name, value))
                },
                _ => {
                    panic!("Unexpected assign")
                }
            }
        },
        Node::AssignOp(op, variable_node, value_node) => {
            let mut initial_value = walk_tree(*variable_node.clone(), scope)?;
            let set_value = walk_tree(*value_node, scope)?;
            match op {
                AssignmentOp::EQ => {
                    initial_value = set_value;
                },
                AssignmentOp::MINUSEQ => {
                    initial_value = CocoValue::CocoNumber(initial_value.as_number() - set_value.as_number());
                },
                AssignmentOp::PLUSEQ => {
                    initial_value = CocoValue::CocoNumber(initial_value.as_number() + set_value.as_number());
                },
                AssignmentOp::MULEQ => {
                    initial_value = CocoValue::CocoNumber(initial_value.as_number() * set_value.as_number());
                },
                AssignmentOp::DIVEQ => {
                    initial_value = CocoValue::CocoNumber(initial_value.as_number() / set_value.as_number());
                },
                AssignmentOp::REMEQ => {
                    initial_value = CocoValue::CocoNumber(initial_value.as_number() % set_value.as_number());
                },
                AssignmentOp::EXPEQ => {
                    initial_value = CocoValue::CocoNumber(initial_value.as_number().powf(set_value.as_number()));
                }
            }

            if let Node::Var(name) = *variable_node.clone() {
               scope.set(name, initial_value.clone());
            }
            if let Node::FieldAccess(var, indices) = *variable_node.clone() {
                if let Node::Var(name) = *var.clone() {
                    let var_value = walk_tree(*var, scope)?;
                    let fields = indices.iter().map(|i| walk_tree(*i.to_owned(), scope).unwrap_or(CocoValue::CocoNull)).collect::<Vec<CocoValue>>();
                    let mut field_accessor = FieldAccessor::new(var_value, fields);
                    let value = field_accessor.set(initial_value, scope);
                    scope.set(name, value);
                }
            }

            Ok(CocoValue::CocoNull)
        },
        Node::Var(name) => Ok(scope.get(name).to_owned()),
        Node::FieldAccess(variable, indices) => {
            let value = walk_tree(*variable, scope)?;
            let fields = indices.iter().map(|i| walk_tree(*i.to_owned(), scope).unwrap_or(CocoValue::CocoNull)).collect::<Vec<CocoValue>>();
            let mut field_accessor = FieldAccessor::new(value, fields);
            Ok(field_accessor.get(scope))
        },
        Node::String(value) => Ok(create_string(value, scope)),
        Node::Number(value) => Ok(CocoValue::CocoNumber(value)),
        Node::Bool(value) => Ok(CocoValue::CocoBoolean(value)),
        Node::Array(value) => {
            let mut array_values = vec![];
            for node in value {
                let value = walk_tree(*node, scope)?;
                array_values.push(Box::new(value))
            }

            Ok(CocoValue::CocoArray(array_values))
        },
        Node::Object(map) => Ok(
            CocoValue::CocoObject(
                map
                .into_iter()
                .map(|x| (x.0, Box::new(walk_tree(*x.1, scope).unwrap())))
                .collect::<BTreeMap<String, Box<CocoValue>>>()
            )
        ),
        Node::Ternary(node, true_cond, false_cond) => {
            let value = walk_tree(*node, scope)?;

            if value.as_bool() {
                return walk_tree(*true_cond, scope);
            }

            walk_tree(*false_cond, scope)
        }
        Node::Logical(operator, node1, node2) => {
            let val1 = walk_tree(*node1, scope);
            let val2 = walk_tree(*node2, scope);

            let ord = val1.clone()?.compare(val2.clone()?);
            
            match operator {
                LogicalOp::AND => Ok(CocoValue::CocoBoolean(val1?.as_bool() && val2?.as_bool())),
                LogicalOp::OR => Ok(CocoValue::CocoBoolean(val1?.as_bool() || val2?.as_bool())),
                LogicalOp::EQ => Ok(CocoValue::CocoBoolean(ord.is_eq())),
                LogicalOp::NOTEQ => Ok(CocoValue::CocoBoolean(ord.is_ne())),
                LogicalOp::GT => Ok(CocoValue::CocoBoolean(ord == Ordering::Greater)),
                LogicalOp::GTEQ => Ok(CocoValue::CocoBoolean(ord == Ordering::Greater || ord == Ordering::Equal)),
                LogicalOp::LT => Ok(CocoValue::CocoBoolean(ord == Ordering::Less)),
                LogicalOp::LTEQ => Ok(CocoValue::CocoBoolean(ord == Ordering::Less || ord == Ordering::Equal))
            }
        },
        Node::Binary(operator, node1, node2) => {
            let val1 = walk_tree(*node1, scope)?;
            let val2 = walk_tree(*node2, scope)?;
            
            match operator {
                BinaryOp::PLUS => {
                    match val1.clone() {
                        CocoValue::CocoString(val) => Ok(CocoValue::CocoString(val + &val2.as_string())),
                        CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val + val2.as_number())),
                        CocoValue::CocoArray(_values) => Ok(CocoValue::CocoString(val1.as_string() + &val2.as_string())),
                        CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() + val2.as_number())),
                        CocoValue::CocoFunction(_n, _a, _b) => Ok(CocoValue::CocoString(val1.as_string() + &val2.as_string())),
                        // FIXME: object + number = string
                        CocoValue::CocoObject(_map) => Ok(CocoValue::CocoString(val1.as_string() + &val2.as_string())),
                        CocoValue::CocoNull => Ok(val2),
                        CocoValue::CocoClass(_n, _p, _c) => Ok(CocoValue::CocoString(val1.as_string() + &val2.as_string()))
                    }
                },
                BinaryOp::MINUS => {
                    match val1.clone() {
                        CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val - val2.as_number())),
                        CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() - val2.as_number())),
                        CocoValue::CocoFunction(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoNull => Ok(CocoValue::CocoNumber(-&val2.as_number())),
                        CocoValue::CocoClass(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                    }
                },
                BinaryOp::MULTIPLY => {
                    match val1.clone() {
                        CocoValue::CocoString(val) => Ok(CocoValue::CocoString(val.repeat(val2.as_number() as usize))),
                        CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val * val2.as_number())),
                        CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() * val2.as_number())),
                        CocoValue::CocoFunction(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0)),
                        CocoValue::CocoClass(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                    }
                },
                BinaryOp::DIVIDE => {
                    match val1.clone() {
                        CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(val1.as_number() / val2.as_number())),
                        CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val / val2.as_number())),
                        CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() / val2.as_number())),
                        CocoValue::CocoFunction(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0)),
                        CocoValue::CocoClass(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                    }
                },
                BinaryOp::REMAINDER => {
                    match val1.clone() {
                        CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(val1.as_number() % val2.as_number())),
                        CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val % val2.as_number())),
                        CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number() % val2.as_number())),
                        CocoValue::CocoFunction(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0)),
                        CocoValue::CocoClass(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                    }
                },
                BinaryOp::EXPONENT => {
                    match val1.clone() {
                        CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(val1.as_number().powf(val2.as_number()))),
                        CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(val.powf(val2.as_number()))),
                        CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(val1.as_number().powf(val2.as_number()))),
                        CocoValue::CocoFunction(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoNull => Ok(CocoValue::CocoNumber(0.0)),
                        CocoValue::CocoClass(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                    }
                }
            }
        },
        Node::Unary(operator, node) => {
            let value = walk_tree(*node, scope)?;

            match operator {
                UnaryOp::MINUS => {
                    match value.clone() {
                        CocoValue::CocoString(_val) => Ok(CocoValue::CocoNumber(-value.as_number())),
                        CocoValue::CocoNumber(val) => Ok(CocoValue::CocoNumber(-val)),
                        CocoValue::CocoArray(_values) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoBoolean(_val) => Ok(CocoValue::CocoNumber(-value.as_number())),
                        CocoValue::CocoFunction(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoObject(_map) => Ok(CocoValue::CocoNumber(f64::NAN)),
                        CocoValue::CocoNull => Ok(CocoValue::CocoNumber(-0.0)),
                        CocoValue::CocoClass(_n, _a, _b) => Ok(CocoValue::CocoNumber(f64::NAN)),
                    }
                },
                UnaryOp::NOT => {
                    Ok(CocoValue::CocoBoolean(!value.as_bool()))
                }
            }
        },
        Node::Fun(variable, args, block) => {
            if let Node::Var(name) = *variable {
                return Ok(scope.set(name.clone(), CocoValue::CocoFunction(name, args, FuncImpl::FromNode(*block))))
            }

            Ok(CocoValue::CocoNull)
        },
        // TODO class and new Class()
        Node::FunCall(variable, args) => {
            let value = walk_tree(*variable, scope)?;
            let mut args_eval = args.iter()
            .map(|arg| walk_tree(*arg.to_owned(), scope).unwrap())
            .collect::<Vec<CocoValue>>();

            match value {
                CocoValue::CocoFunction(_n, mut fun_args, fun_block) => {
                    let reduced_args = fun_args.reduce(&mut args_eval);

                    match fun_block {
                        FuncImpl::FromNode(block) => {
                            let mut fun_scope = Scope::new(Some(Box::new(scope.to_owned())));

                            for arg in reduced_args {
                                fun_scope.set(arg.0, arg.1);
                            }

                            walk_tree(block, &mut fun_scope)
                        },
                        FuncImpl::Builtin(f) => {
                            Ok(f(reduced_args))
                        }
                    }
                    
                },
                _ => {
                    Err("Unknown function".to_string())
                }
            }
        },
        Node::SwitchStatement(variable, switch_cases) => {
            let value = walk_tree(*variable, scope);

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
                                        let next_default_statement_value = walk_tree(next_default_statement.to_owned(), scope);

                                        //println!("{:#?}", next_default_statement);

                                        return next_default_statement_value;
                                    },
                                    SwitchCase::Case(next_val, next_statement) => {
                                        if next_statement.is_none() {
                                            continue;
                                        }

                                        let next_val_value = walk_tree(next_val.to_owned(), scope);
                                        let next_statement_value = walk_tree(next_statement.to_owned().unwrap(), scope);

                                        if next_val_value == value {
                                            return next_statement_value
                                        }

                                        continue;
                                    }
                                }
                            } 
                        }

                        let node_val = walk_tree(val.to_owned(), scope);
                        let statement_value = walk_tree(statement.to_owned().unwrap(), scope);
                        if node_val == value {
                            return statement_value
                        }

                        continue;
                    },
                    SwitchCase::Default(statement) => {
                        let statement_value = walk_tree(statement.to_owned(), scope);

                        return statement_value
                    }
                }
            }
        },
        Node::IfElseStatement(cond, if_node, else_node) => {
            // FIXME: scope?
            if walk_tree(*cond, scope)?.as_bool() {
                return walk_tree(*if_node, scope)
            }

            if else_node.is_none() {
                return Ok(CocoValue::CocoNull)
            }

            walk_tree(else_node.unwrap(), scope)
        },
        Node::WhileStatement(cond, node) => {
            while walk_tree(*cond.clone(), scope)?.as_bool() {
                walk_tree(*node.clone(), scope);
            }

            Ok(CocoValue::CocoNull)
        }
        _ => Ok(CocoValue::CocoNull)
    }
}