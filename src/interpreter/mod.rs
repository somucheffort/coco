use core::panic;
use std::{collections::{BTreeMap}, cmp::Ordering};

use crate::{parser::{ Node, SwitchCase, LogicalOp, BinaryOp, UnaryOp, AssignmentOp }, modules::import_module, Error};

pub mod scope;
pub mod types;

use self::{scope::{ Scope }, types::{Value, FieldAccessor, FuncImpl}};

pub struct Interpreter {}

pub fn walk_tree(node: Node, scope: &mut Scope) -> Result<Value, Error> {
    match node {
        Node::Import(module) => {
            import_module(module.as_str(), scope, None);
            Ok(Value::Null)
        },
        Node::ImportFrom(module, objects) => {
            import_module(module.as_str(), scope, Some(objects));
            Ok(Value::Null)
        },
        Node::BlockStatement(statements) => {
            let mut result = Value::Null;

            

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
                    initial_value = Value::Number(initial_value.as_number() - set_value.as_number());
                },
                AssignmentOp::PLUSEQ => {
                    initial_value = match initial_value.clone() {
                        Value::String(_) => Value::String(initial_value.as_string() + &set_value.as_string()),
                        _ => Value::Number(initial_value.as_number() + set_value.as_number())
                    }
                },
                AssignmentOp::MULEQ => {
                    initial_value = Value::Number(initial_value.as_number() * set_value.as_number());
                },
                AssignmentOp::DIVEQ => {
                    initial_value = Value::Number(initial_value.as_number() / set_value.as_number());
                },
                AssignmentOp::REMEQ => {
                    initial_value = Value::Number(initial_value.as_number() % set_value.as_number());
                },
                AssignmentOp::EXPEQ => {
                    initial_value = Value::Number(initial_value.as_number().powf(set_value.as_number()));
                }
            }

            if let Node::Var(name) = *variable_node.clone() {
                scope.set(name, initial_value.clone());
            }

            if let Node::FieldAccess(var, indices) = *variable_node {
                if let Node::Var(name) = *var.clone() {
                    let var_value = walk_tree(*var, scope)?;
                    let fields = indices.iter().map(|i| walk_tree(*i.to_owned(), scope).unwrap_or(Value::Null)).collect::<Vec<Value>>();
                    let mut field_accessor = FieldAccessor::new(var_value, fields);
                    let value = field_accessor.set(initial_value, scope);

                    scope.set(name, value);
                }
            }

            Ok(Value::Null)
        },
        Node::Var(name) => Ok(scope.get(name).to_owned()),
        Node::FieldAccess(variable, indices) => {
            let value = walk_tree(*variable, scope)?;
            let fields = indices.iter().map(|i| walk_tree(*i.to_owned(), scope).unwrap_or(Value::Null)).collect::<Vec<Value>>();
            let mut field_accessor = FieldAccessor::new(value, fields);
            Ok(field_accessor.get(scope))
        },
        Node::String(value) => Ok(Value::create_string(value, scope)),
        Node::Number(value) => Ok(Value::Number(value)),
        Node::Bool(value) => Ok(Value::Boolean(value)),
        Node::Array(value) => {
            let mut array_values = vec![];
            for node in value {
                let value = walk_tree(*node, scope)?;
                array_values.push(Box::new(value))
            }

            Ok(Value::Array(array_values))
        },
        Node::Object(map) => Ok(
            Value::Object(
                map
                .into_iter()
                .map(|x| (x.0, Box::new(walk_tree(*x.1, scope).unwrap())))
                .collect::<BTreeMap<String, Box<Value>>>()
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
                LogicalOp::AND => Ok(Value::Boolean(val1?.as_bool() && val2?.as_bool())),
                LogicalOp::OR => Ok(Value::Boolean(val1?.as_bool() || val2?.as_bool())),
                LogicalOp::EQ => Ok(Value::Boolean(ord.is_eq())),
                LogicalOp::NOTEQ => Ok(Value::Boolean(ord.is_ne())),
                LogicalOp::GT => Ok(Value::Boolean(ord == Ordering::Greater)),
                LogicalOp::GTEQ => Ok(Value::Boolean(ord == Ordering::Greater || ord == Ordering::Equal)),
                LogicalOp::LT => Ok(Value::Boolean(ord == Ordering::Less)),
                LogicalOp::LTEQ => Ok(Value::Boolean(ord == Ordering::Less || ord == Ordering::Equal))
            }
        },
        Node::Binary(operator, node1, node2) => {
            let val1 = walk_tree(*node1, scope)?;
            let val2 = walk_tree(*node2, scope)?;
            
            match operator {
                BinaryOp::PLUS => {
                    match val1.clone() {
                        Value::String(val) => Ok(Value::String(val + &val2.as_string())),
                        Value::Number(val) => Ok(Value::Number(val + val2.as_number())),
                        Value::Array(_values) => Ok(Value::String(val1.as_string() + &val2.as_string())),
                        Value::Boolean(_val) => Ok(Value::Number(val1.as_number() + val2.as_number())),
                        Value::Function(_n, _a, _b) => Ok(Value::String(val1.as_string() + &val2.as_string())),
                        // FIXME: object + number = string
                        Value::Object(_map) => Ok(Value::String(val1.as_string() + &val2.as_string())),
                        Value::Null => Ok(val2),
                        Value::Class(_n, _p, _c) => Ok(Value::String(val1.as_string() + &val2.as_string()))
                    }
                },
                BinaryOp::MINUS => {
                    match val1.clone() {
                        Value::String(_val) => Ok(Value::Number(f64::NAN)),
                        Value::Number(val) => Ok(Value::Number(val - val2.as_number())),
                        Value::Array(_values) => Ok(Value::Number(f64::NAN)),
                        Value::Boolean(_val) => Ok(Value::Number(val1.as_number() - val2.as_number())),
                        Value::Function(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                        Value::Object(_map) => Ok(Value::Number(f64::NAN)),
                        Value::Null => Ok(Value::Number(-&val2.as_number())),
                        Value::Class(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                    }
                },
                BinaryOp::MULTIPLY => {
                    match val1.clone() {
                        Value::String(val) => Ok(Value::String(val.repeat(val2.as_number() as usize))),
                        Value::Number(val) => Ok(Value::Number(val * val2.as_number())),
                        Value::Array(_values) => Ok(Value::Number(f64::NAN)),
                        Value::Boolean(_val) => Ok(Value::Number(val1.as_number() * val2.as_number())),
                        Value::Function(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                        Value::Object(_map) => Ok(Value::Number(f64::NAN)),
                        Value::Null => Ok(Value::Number(0.0)),
                        Value::Class(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                    }
                },
                BinaryOp::DIVIDE => {
                    match val1.clone() {
                        Value::String(_val) => Ok(Value::Number(val1.as_number() / val2.as_number())),
                        Value::Number(val) => Ok(Value::Number(val / val2.as_number())),
                        Value::Array(_values) => Ok(Value::Number(f64::NAN)),
                        Value::Boolean(_val) => Ok(Value::Number(val1.as_number() / val2.as_number())),
                        Value::Function(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                        Value::Object(_map) => Ok(Value::Number(f64::NAN)),
                        Value::Null => Ok(Value::Number(0.0)),
                        Value::Class(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                    }
                },
                BinaryOp::REMAINDER => {
                    match val1.clone() {
                        Value::String(_val) => Ok(Value::Number(val1.as_number() % val2.as_number())),
                        Value::Number(val) => Ok(Value::Number(val % val2.as_number())),
                        Value::Array(_values) => Ok(Value::Number(f64::NAN)),
                        Value::Boolean(_val) => Ok(Value::Number(val1.as_number() % val2.as_number())),
                        Value::Function(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                        Value::Object(_map) => Ok(Value::Number(f64::NAN)),
                        Value::Null => Ok(Value::Number(0.0)),
                        Value::Class(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                    }
                },
                BinaryOp::EXPONENT => {
                    match val1.clone() {
                        Value::String(_val) => Ok(Value::Number(val1.as_number().powf(val2.as_number()))),
                        Value::Number(val) => Ok(Value::Number(val.powf(val2.as_number()))),
                        Value::Array(_values) => Ok(Value::Number(f64::NAN)),
                        Value::Boolean(_val) => Ok(Value::Number(val1.as_number().powf(val2.as_number()))),
                        Value::Function(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                        Value::Object(_map) => Ok(Value::Number(f64::NAN)),
                        Value::Null => Ok(Value::Number(0.0)),
                        Value::Class(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                    }
                }
            }
        },
        Node::Unary(operator, node) => {
            let value = walk_tree(*node, scope)?;

            match operator {
                UnaryOp::MINUS => {
                    match value.clone() {
                        Value::String(_val) => Ok(Value::Number(-value.as_number())),
                        Value::Number(val) => Ok(Value::Number(-val)),
                        Value::Array(_values) => Ok(Value::Number(f64::NAN)),
                        Value::Boolean(_val) => Ok(Value::Number(-value.as_number())),
                        Value::Function(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                        Value::Object(_map) => Ok(Value::Number(f64::NAN)),
                        Value::Null => Ok(Value::Number(-0.0)),
                        Value::Class(_n, _a, _b) => Ok(Value::Number(f64::NAN)),
                    }
                },
                UnaryOp::NOT => {
                    Ok(Value::Boolean(!value.as_bool()))
                }
            }
        },
        Node::Fun(variable, args, block) => {
            if let Node::Var(name) = *variable {
                return Ok(scope.set(
                    name.clone(), 
                    Value::Function(name, args, FuncImpl::FromNode(*block))
                ))
            }

            Ok(Value::Null)
        },
        // TODO class and new Class()
        Node::Class(name, constructor, prototype) => {
            println!("{:#?}", name);
            
            let prot = prototype.iter().fold(BTreeMap::default(), |mut acc, val| {
                let fun = walk_tree(val.1.to_owned(), scope).unwrap();

                acc.insert(val.0.to_owned(), Box::new(fun));

                acc
            });

            let cons: Option<Box<Value>> = constructor.map(|c| Box::new(walk_tree(*c, scope).unwrap()));

            // fixme
            Ok(scope.set(name.clone(), Value::Class(name, cons, prot)))
        },
        Node::FunCall(variable, args) => {
            let value = walk_tree(*variable.clone(), scope)?;
            let mut args_eval = args.iter()
            .map(|arg| walk_tree(*arg.to_owned(), scope).unwrap())
            .collect::<Vec<Value>>();

            match value {
                Value::Function(_, mut fun_args, fun_block) => {
                    let reduced_args = fun_args.reduce(&mut args_eval);

                    match fun_block {
                        FuncImpl::FromNode(block) => {
                            let mut fun_scope = Scope::from(Some(Box::new(scope.to_owned())), scope.filename.clone());

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
                    match *variable {
                        Node::Var(name) => {
                            scope.throw_exception(format!("{name} is not a function"), vec![0, 0]);
                            return Err(Error { msg: "".to_string(), pos: vec![] })
                        },
                        Node::FieldAccess(var, _) => {
                            if let Node::Var(name) = *var {
                                scope.throw_exception(format!("{name} is not a function"), vec![0, 0]);
                                return Err(Error { msg: "".to_string(), pos: vec![] })
                            }
                        },
                        _ => {}
                    }

                    scope.throw_exception("undefined is not a function".to_string(), vec![0, 0]);
                    Err(Error { msg: "".to_string(), pos: vec![] })
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
            // FIXME: stack?
            if walk_tree(*cond, scope)?.as_bool() {
                return walk_tree(*if_node, scope)
            }

            if else_node.is_none() {
                return Ok(Value::Null)
            }

            walk_tree(else_node.unwrap(), scope)
        },
        Node::WhileStatement(cond, node) => {
            while walk_tree(*cond.clone(), scope)?.as_bool() {
                walk_tree(*node.clone(), scope);
            }

            Ok(Value::Null)
        },
        Node::ForStatement(variable, iterator, block) => {
            let iter = walk_tree(*iterator, scope)?;

            match &iter {
                Value::String(str) => {
                    let str_splitted = str
                        .chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<Value>>();

                    for value in str_splitted {
                        scope.set(variable.clone(), value);
                        walk_tree(*block.clone(), scope);
                    }

                    Ok(Value::Null)
                },
                Value::Array(values) => {
                    let values_unboxed = values.iter().map(|val| *val.to_owned()).collect::<Vec<Value>>();
                    for value in values_unboxed {
                        scope.set(variable.clone(), value);
                        walk_tree(*block.clone(), scope);
                    }

                    Ok(Value::Null)
                },
                _ => {
                    scope.throw_exception("Value cannot be iterated".to_string(), vec![0, 0]);
                    Err(Error { msg: "Value cannot be iterated".to_string(), pos: vec![0, 0] })
                }
            }
        },
        Node::Range(from, to, inclusive) => {
            let from_value = walk_tree(*from, scope)?.as_number();
            let to_value = walk_tree(*from, scope)?.as_number();


            if inclusive {

                let b = vec![(from_value..to_value)];
                return Ok(Value::Array(
                    
                ))
            }

            Ok(Value::Array())
        },
        _ => Ok(Value::Null)
    }
}