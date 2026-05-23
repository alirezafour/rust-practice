pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

use crate::parser::{Expr, Stmt, Token, TokenTypes};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    fn get_cloned(&self, key: &str) -> Option<LoxValue> {
        match self.map.get(key) {
            Some(value) => Some(value.clone()),
            None => {
                return match &self.parent {
                    Some(parent) => parent.borrow().get_cloned(key),
                    None => None,
                };
            }
        }
    }
    fn set(&mut self, key: &str, value: LoxValue) -> bool {
        match self.map.get_mut(key) {
            Some(v) => {
                *v = value;
                true
            }
            None => {
                return match &self.parent {
                    Some(parent) => parent.borrow_mut().set(key, value),
                    None => false,
                };
            }
        }
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment {
                map: HashMap::new(),
                parent: None,
            })),
        }
    }
    fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue, RuntimeError> {
        match expr {
            Expr::Literal { identifier } => match identifier.as_str() {
                "true" => return Ok(LoxValue::Bool(true)),
                "false" => return Ok(LoxValue::Bool(false)),
                "nil" => return Ok(LoxValue::Nil),
                _ => {
                    // check for string
                    if identifier.starts_with("\"") {
                        return Ok(LoxValue::String(
                            identifier[1..identifier.len() - 1].to_string(),
                        ));
                    // check for number
                    } else if let Ok(num) = identifier.parse::<f64>() {
                        return Ok(LoxValue::Number(num));
                    // check identifier
                    } else if let Some(val) = self.environment.borrow().get_cloned(identifier) {
                        return Ok(val.clone());
                    } else {
                        return Err(RuntimeError {
                            token: Token {
                                token_type: TokenTypes::Identifier,
                                lexeme: identifier.into(),
                                line: 0,
                                column: 0,
                            },
                            message: format!("identifier `{identifier}` not found"),
                        });
                    }
                }
            },
            Expr::Binary {
                left,
                operation,
                right,
            } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                return self.binary_eval(&left_val, &right_val, &operation);
            }
            Expr::Unary { operation, right } => {
                let right_val = self.evaluate(right)?;
                match (&right_val, &operation.token_type) {
                    (LoxValue::Number(val), TokenTypes::Minus) => {
                        return Ok(LoxValue::Number(-val));
                    }
                    (_, TokenTypes::Bang) => {
                        return Ok(LoxValue::Bool(!self.is_truthy(&right_val)));
                    }
                    _ => {
                        return Err(RuntimeError {
                            token: operation.clone(),
                            message: "expected right side to be number or bool.".into(),
                        });
                    }
                }
            }
            Expr::Grouping { expression } => {
                return self.evaluate(expression);
            }
            Expr::Assign { identifier, right } => {
                let right_val = self.evaluate(right)?;
                if !self
                    .environment
                    .borrow_mut()
                    .set(identifier, right_val.clone())
                {
                    return Err(RuntimeError {
                        token: Token {
                            token_type: TokenTypes::Identifier,
                            lexeme: format!("{identifier}"),
                            line: 0,
                            column: 0,
                        },
                        message: "identifier {identifier} not defined.".into(),
                    });
                }
                return Ok(right_val);
            }
            Expr::Logical {
                left,
                logical,
                right,
            } => {
                let left_val = self.evaluate(left)?;
                let bool_left = self.is_truthy(&left_val);
                match (bool_left, &logical.token_type) {
                    (true, TokenTypes::Or) | (false, TokenTypes::And) => return Ok(left_val),
                    (true, TokenTypes::And) | (false, TokenTypes::Or) => {
                        return self.evaluate(right);
                    }
                    _ => {
                        return Err(RuntimeError {
                            token: logical.clone(),
                            message: "invalid logical operation.".into(),
                        });
                    }
                }
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let identifier = self.evaluate(callee)?;
                match identifier {
                    LoxValue::Function {
                        name,
                        parameters,
                        body,
                    } => {
                        if arguments.len() != parameters.len() {
                            return Err(RuntimeError {
                                token: Token {
                                    token_type: TokenTypes::Identifier,
                                    lexeme: "{identifier}".into(),
                                    line: 0,
                                    column: 0,
                                },
                                message: format!(
                                    "function {name} expected {} params.",
                                    parameters.len()
                                ),
                            });
                        }
                        let old_env = Rc::clone(&self.environment);
                        let fun_env = Rc::new(RefCell::new(Environment {
                            map: HashMap::new(),
                            parent: Some(Rc::clone(&self.environment)),
                        }));
                        for (pram_name, arg_expr) in parameters.iter().zip(arguments.iter()) {
                            let value = self.evaluate(arg_expr)?;
                            fun_env.borrow_mut().map.insert(pram_name.clone(), value);
                        }
                        self.environment = fun_env;
                        let result = self.execute(&body)?;
                        self.environment = old_env;
                        Ok(result.unwrap_or(LoxValue::Nil))
                    }
                    _ => Err(RuntimeError {
                        token: paren.clone(),
                        message: "expected function.".into(),
                    }),
                }
            }
            Expr::Lambda { params, body } => Ok(LoxValue::Function {
                name: String::new(),
                parameters: params.clone(),
                body: body.clone(),
            }),
            _ => Err(RuntimeError {
                token: Token {
                    token_type: TokenTypes::Eof,
                    lexeme: "Eof".into(),
                    line: 0,
                    column: 0,
                },
                message: "expression not supported yet".into(),
            }),
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<Option<LoxValue>, RuntimeError> {
        match stmt {
            Stmt::Var { name, value } => {
                let val = value
                    .as_ref()
                    .map(|v| self.evaluate(v))
                    .transpose()?
                    .unwrap_or(LoxValue::Nil);
                self.environment
                    .borrow_mut()
                    .map
                    .insert(name.to_string(), val);
                Ok(None)
            }
            Stmt::Print { expr } => {
                let output = self.evaluate(expr)?;
                println!("{}", output);
                Ok(None)
            }
            Stmt::Expression { expr } => {
                self.evaluate(expr)?;
                Ok(None)
            }
            Stmt::If {
                condition,
                body,
                else_branch,
            } => {
                let res = self.evaluate(condition)?;
                if self.is_truthy(&res) {
                    self.execute(body)
                } else {
                    match else_branch {
                        Some(body) => self.execute(body),
                        None => Ok(None),
                    }
                }
            }
            Stmt::While { condition, body } => {
                let mut res = self.evaluate(condition)?;
                let mut result = None;
                while self.is_truthy(&res) {
                    result = self.execute(body)?;
                    if result.is_some() {
                        break;
                    }
                    res = self.evaluate(condition)?;
                }
                Ok(result)
            }
            Stmt::Block { data } => {
                let old_env = Rc::clone(&self.environment);
                let fun_env = Rc::new(RefCell::new(Environment {
                    map: HashMap::new(),
                    parent: Some(Rc::clone(&self.environment)),
                }));
                self.environment = fun_env;
                let mut out = None;
                for stmt in data {
                    out = self.execute(stmt)?;
                    if out.is_some() {
                        break;
                    }
                }
                self.environment = old_env;
                Ok(out)
            }
            Stmt::Function { name, params, body } => {
                self.environment.borrow_mut().map.insert(
                    name.clone(),
                    LoxValue::Function {
                        name: name.clone(),
                        parameters: params.clone(),
                        body: body.clone(),
                    },
                );
                Ok(None)
            }
            Stmt::Return { value } => {
                let result = match value {
                    Some(v) => self.evaluate(v)?,
                    None => LoxValue::Nil,
                };
                Ok(Some(result))
            }
            _ => Err(RuntimeError {
                token: Token {
                    token_type: TokenTypes::Eof,
                    lexeme: "Eof".into(),
                    line: 0,
                    column: 0,
                },
                message: "I didn't like it".into(),
            }),
        }
    }

    fn is_truthy(&self, val: &LoxValue) -> bool {
        match val {
            LoxValue::Nil => false,
            LoxValue::Bool(v) => *v,
            _ => true,
        }
    }

    fn binary_eval(
        &self,
        left: &LoxValue,
        right: &LoxValue,
        op: &Token,
    ) -> Result<LoxValue, RuntimeError> {
        match op.token_type {
            TokenTypes::Greater
            | TokenTypes::Less
            | TokenTypes::LessEqual
            | TokenTypes::GreaterEqual => return self.comparison_eval(left, right, &op),
            TokenTypes::Equal | TokenTypes::BangEqual => self.equality_eval(left, right, &op),
            TokenTypes::Plus | TokenTypes::Minus | TokenTypes::Star | TokenTypes::Slash => {
                self.arithmetic_eval(left, right, &op)
            }
            _ => Err(RuntimeError {
                token: op.clone(),
                message: "invalid token type.".into(),
            }),
        }
    }
    fn comparison_eval(
        &self,
        left: &LoxValue,
        right: &LoxValue,
        op: &Token,
    ) -> Result<LoxValue, RuntimeError> {
        let token_type = &op.token_type;
        match (left, right, token_type) {
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Greater) => {
                Ok(LoxValue::Bool(a > b))
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::GreaterEqual) => {
                Ok(LoxValue::Bool(a >= b))
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Less) => {
                Ok(LoxValue::Bool(a < b))
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::LessEqual) => {
                Ok(LoxValue::Bool(a <= b))
            }
            _ => Err(RuntimeError {
                token: op.clone(),
                message: "unexpected call for comparison eval.".into(),
            }),
        }
    }
    fn equality_eval(
        &self,
        left: &LoxValue,
        right: &LoxValue,
        op: &Token,
    ) -> Result<LoxValue, RuntimeError> {
        match op.token_type {
            TokenTypes::EqualEqual => Ok(LoxValue::Bool(left == right)),
            TokenTypes::BangEqual => Ok(LoxValue::Bool(left != right)),
            _ => Err(RuntimeError {
                token: op.clone(),
                message: "wrong call/type to equality.".into(),
            }),
        }
    }
    fn arithmetic_eval(
        &self,
        left: &LoxValue,
        right: &LoxValue,
        op: &Token,
    ) -> Result<LoxValue, RuntimeError> {
        let token_type = &op.token_type;
        match (left, right, token_type) {
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Plus) => {
                Ok(LoxValue::Number(a + b))
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Minus) => {
                Ok(LoxValue::Number(a - b))
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Star) => {
                Ok(LoxValue::Number(a * b))
            }
            (LoxValue::Number(a), LoxValue::Number(b), TokenTypes::Slash) => {
                Ok(LoxValue::Number(a / b))
            }
            (LoxValue::String(a), LoxValue::String(b), TokenTypes::Plus) => {
                let mut new = String::from(a);
                new.push_str(b);
                Ok(LoxValue::String(new))
            }
            _ => Err(RuntimeError {
                token: op.clone(),
                message: "wrong call/type to arithmetic.".into(),
            }),
        }
    }
}
