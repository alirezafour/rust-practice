use crate::parser::{Expr, Stmt, Token, TokenTypes};
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum LoxValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<Stmt>,
    },
}

impl std::fmt::Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoxValue::Bool(v) => {
                if *v {
                    write!(f, "true")?
                } else {
                    write!(f, "false")?
                }
                Ok(())
            }
            LoxValue::Number(n) => {
                write!(f, "{}", n)?;
                Ok(())
            }
            LoxValue::String(s) => {
                write!(f, "{}", s)?;
                Ok(())
            }
            LoxValue::Nil => {
                write!(f, "nil")?;
                Ok(())
            }
            LoxValue::Function {
                name,
                parameters,
                body,
            } => {
                write!(f, "Fun {}({:?})", name, parameters)?;
                Ok(())
            }
            _ => Err(std::fmt::Error),
        }
    }
}

struct Environment {
    map: HashMap<String, LoxValue>,
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment {
                map: HashMap::new(),
            },
        }
    }
    fn evaluate(&mut self, expr: &Expr) -> LoxValue {
        match expr {
            Expr::Literal { identifier } => match identifier.as_str() {
                "true" => return LoxValue::Bool(true),
                "false" => return LoxValue::Bool(false),
                "nil" => return LoxValue::Nil,
                _ => {
                    // check for string
                    if identifier.starts_with("\"") {
                        return LoxValue::String(identifier[1..identifier.len() - 1].to_string());
                    // check for number
                    } else if let Ok(num) = identifier.parse::<f64>() {
                        return LoxValue::Number(num);
                    // check identifier
                    } else if let Some(val) = self.environment.map.get(identifier) {
                        return val.clone();
                    } else {
                        panic!("identifier not found");
                    }
                }
            },
            Expr::Binary {
                left,
                operation,
                right,
            } => {
                let left_val = self.evaluate(left);
                let right_val = self.evaluate(right);
                return self.binary_eval(&left_val, &right_val, &operation);
            }
            Expr::Unary { operation, right } => {
                let right_val = self.evaluate(right);
                match (&right_val, &operation.token_type) {
                    (LoxValue::Number(val), TokenTypes::Minus) => return LoxValue::Number(-val),
                    (_, TokenTypes::Bang) => return LoxValue::Bool(!self.is_truthy(&right_val)),
                    _ => panic!("right side is invalid."),
                }
            }
            Expr::Grouping { expression } => {
                return self.evaluate(expression);
            }
            Expr::Assign { identifier, right } => {
                let right_val = self.evaluate(right);
                let iter = self.environment.map.get_mut(identifier);
                match iter {
                    Some(v) => *v = right_val.clone(),
                    None => panic!("identifier not defined."),
                }
                return right_val;
            }
            Expr::Logical {
                left,
                logical,
                right,
            } => {
                let left_val = self.evaluate(left);
                let bool_left = self.is_truthy(&left_val);
                match (bool_left, &logical.token_type) {
                    (true, TokenTypes::Or) | (false, TokenTypes::And) => return left_val,
                    (true, TokenTypes::And) | (false, TokenTypes::Or) => {
                        let right_val = self.evaluate(right);
                        return right_val;
                    }
                    _ => panic!("abc"),
                }
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let identifier = self.evaluate(callee);
                match identifier {
                    LoxValue::Function {
                        name,
                        parameters,
                        body,
                    } => {
                        if arguments.len() != parameters.len() {
                            panic!("wrong param count");
                        }
                        let mut fun_env = HashMap::new();
                        for (pram_name, arg_expr) in parameters.iter().zip(arguments.iter()) {
                            let value = self.evaluate(arg_expr);
                            fun_env.insert(pram_name.clone(), value);
                        }
                        let old_env = self.environment.map.clone();
                        self.environment.map = fun_env;
                        let result = self.execute(&body);
                        self.environment.map = old_env;
                        result.unwrap_or(LoxValue::Nil)
                    }
                    _ => panic!("wrong callee"),
                }
            }
            Expr::Lambda { params, body } => LoxValue::Function {
                name: String::new(),
                parameters: params.clone(),
                body: body.clone(),
            },
            _ => {
                panic!("not supported yet");
            }
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Option<LoxValue> {
        match stmt {
            Stmt::Var { name, value } => {
                let val = value
                    .as_ref()
                    .map(|v| self.evaluate(v))
                    .unwrap_or(LoxValue::Nil);
                self.environment.map.insert(name.to_string(), val);
                None
            }
            Stmt::Print { expr } => {
                println!("{}", self.evaluate(expr));
                None
            }
            Stmt::Expression { expr } => {
                self.evaluate(expr);
                None
            }
            Stmt::If {
                condition,
                body,
                else_branch,
            } => {
                let res = self.evaluate(condition);
                if self.is_truthy(&res) {
                    let result = self.execute(body);
                    result
                } else {
                    match else_branch {
                        Some(body) => {
                            let result = self.execute(body);
                            result
                        }
                        None => None,
                    }
                }
            }
            Stmt::While { condition, body } => {
                let mut res = self.evaluate(condition);
                let mut result = None;
                while self.is_truthy(&res) {
                    result = self.execute(body);
                    if result.is_some() {
                        break;
                    }
                    res = self.evaluate(condition);
                }
                result
            }
            Stmt::Block { data } => {
                let old_env = self.environment.map.clone();
                let mut out = None;
                for stmt in data {
                    out = self.execute(stmt);
                    if out.is_some() {
                        break;
                    }
                }
                self.environment.map = old_env;
                out
            }
            Stmt::Function { name, params, body } => {
                self.environment.map.insert(
                    name.clone(),
                    LoxValue::Function {
                        name: name.clone(),
                        parameters: params.clone(),
                        body: body.clone(),
                    },
                );
                None
            }
            Stmt::Return { value } => {
                let result = match value {
                    Some(v) => self.evaluate(v),
                    None => LoxValue::Nil,
                };
                Some(result)
            }
            _ => panic!("I didn't like it"),
        }
    }

    fn is_truthy(&self, val: &LoxValue) -> bool {
        match val {
            LoxValue::Nil => false,
            LoxValue::Bool(v) => *v,
            _ => true,
        }
    }

    fn binary_eval(&self, left: &LoxValue, right: &LoxValue, op: &Token) -> LoxValue {
        match op.token_type {
            TokenTypes::Greater
            | TokenTypes::Less
            | TokenTypes::LessEqual
            | TokenTypes::GreaterEqual => return self.comparison_eval(left, right, &op.token_type),
            TokenTypes::Equal | TokenTypes::BangEqual => {
                return self.equality_eval(left, right, &op.token_type);
            }
            TokenTypes::Plus | TokenTypes::Minus | TokenTypes::Star | TokenTypes::Slash => {
                return self.arithmetic_eval(left, right, &op.token_type);
            }
            _ => panic!("invalid operation."),
        }
    }
    fn comparison_eval(&self, left: &LoxValue, right: &LoxValue, op: &TokenTypes) -> LoxValue {
        match (left, right, op) {
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Greater) => {
                return LoxValue::Bool(a > b);
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::GreaterEqual) => {
                return LoxValue::Bool(a >= b);
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Less) => {
                return LoxValue::Bool(a < b);
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::LessEqual) => {
                return LoxValue::Bool(a <= b);
            }
            _ => panic!("wrong call/type to comparison."),
        }
    }
    fn equality_eval(&self, left: &LoxValue, right: &LoxValue, op: &TokenTypes) -> LoxValue {
        match op {
            TokenTypes::EqualEqual => {
                return LoxValue::Bool(left == right);
            }
            TokenTypes::BangEqual => {
                return LoxValue::Bool(left != right);
            }
            _ => panic!("wrong call/type to equality."),
        }
    }
    fn arithmetic_eval(&self, left: &LoxValue, right: &LoxValue, op: &TokenTypes) -> LoxValue {
        match (left, right, op) {
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Plus) => {
                return LoxValue::Number(a + b);
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Minus) => {
                return LoxValue::Number(a - b);
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Star) => {
                return LoxValue::Number(a * b);
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Slash) => {
                return LoxValue::Number(a / b);
            }
            (LoxValue::String(a), LoxValue::String(b), TokenTypes::Plus) => {
                let mut new = String::from(a);
                new.push_str(b);
                return LoxValue::String(new);
            }
            _ => panic!("wrong call/type to arithmetic."),
        }
    }
}
