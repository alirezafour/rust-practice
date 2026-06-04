use crate::scanner::{Expr, Stmt, Token, TokenTypes};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line: {}, column: {}] [Token: {}] Parser Error: {}",
            self.token.line, self.token.column, self.token, self.message
        )
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoxValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<Stmt>,
        env: Rc<RefCell<Environment>>,
    },
    Class {
        name: String,
        methods: HashMap<String, LoxValue>,
        superclass: Option<Token>,
    },
    Instance {
        fields: Rc<RefCell<HashMap<String, LoxValue>>>,
        class_name: String,
    },
}

impl std::fmt::Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoxValue::Bool(v) => write!(f, "{}", v),
            LoxValue::Number(n) => write!(f, "{}", n),
            LoxValue::String(s) => write!(f, "{}", s),
            LoxValue::Nil => write!(f, "nil"),
            LoxValue::Function {
                name, parameters, ..
            } => write!(f, "Fun {}({:?})", name, parameters),
            LoxValue::Class { name, .. } => write!(f, "Class {}()", name),
            LoxValue::Instance { class_name, .. } => write!(f, "Instance {}", class_name),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Environment {
    map: HashMap<String, LoxValue>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    fn get_cloned(&self, key: &str) -> Option<LoxValue> {
        match self.map.get(key) {
            Some(value) => Some(value.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().get_cloned(key),
                None => None,
            },
        }
    }
    fn set(&mut self, key: &str, value: LoxValue) -> bool {
        match self.map.get_mut(key) {
            Some(v) => {
                *v = value;
                true
            }
            None => match &self.parent {
                Some(parent) => parent.borrow_mut().set(key, value),
                None => false,
            },
        }
    }
    fn set_field(&mut self, key: &str, field: &str, value: LoxValue) -> bool {
        match self.map.get_mut(key) {
            Some(v) => match v {
                LoxValue::Instance { fields, .. } => {
                    fields.borrow_mut().insert(field.into(), value);
                    true
                }
                _ => false,
            },
            None => match &self.parent {
                Some(paren) => paren.borrow_mut().set_field(key, field, value),
                None => false,
            },
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
            Expr::Literal { identifier } => match &identifier.token_type {
                TokenTypes::True => Ok(LoxValue::Bool(true)),
                TokenTypes::False => Ok(LoxValue::Bool(false)),
                TokenTypes::Nil => Ok(LoxValue::Nil),
                TokenTypes::String => Ok(LoxValue::String(identifier.lexeme.clone())),
                TokenTypes::Number => match identifier.lexeme.parse::<f64>() {
                    Ok(v) => Ok(LoxValue::Number(v)),
                    Err(e) => Err(RuntimeError {
                        token: identifier.clone(),
                        message: format!("invalid number. error: {e}"),
                    }),
                }, // remove unwrap use match
                TokenTypes::Identifier => {
                    match self.environment.borrow().get_cloned(&identifier.lexeme) {
                        Some(v) => Ok(v),
                        None => Err(RuntimeError {
                            token: identifier.clone(),
                            message: format!("identifier `{identifier}` not found"),
                        }),
                    }
                }
                TokenTypes::This => {
                    // Look up "this" in the environment (bound by method calls)
                    match self.environment.borrow().get_cloned("this") {
                        Some(v) => Ok(v),
                        None => Err(RuntimeError {
                            token: identifier.clone(),
                            message: "this can only be used inside methods.".into(),
                        }),
                    }
                }
                _ => Err(RuntimeError {
                    token: identifier.clone(),
                    message: "invalid literals".into(),
                }),
            },
            Expr::Binary {
                left,
                operation,
                right,
            } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.binary_eval(&left_val, &right_val, &operation)
            }
            Expr::Unary { operation, right } => {
                let right_val = self.evaluate(right)?;
                match (&right_val, &operation.token_type) {
                    (LoxValue::Number(val), TokenTypes::Minus) => Ok(LoxValue::Number(-val)),
                    (_, TokenTypes::Bang) => Ok(LoxValue::Bool(!self.is_truthy(&right_val))),
                    _ => Err(RuntimeError {
                        token: operation.clone(),
                        message: "expected right side to be number or bool.".into(),
                    }),
                }
            }
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Assign { identifier, right } => {
                let right_val = self.evaluate(right)?;
                if !self
                    .environment
                    .borrow_mut()
                    .set(&identifier.lexeme, right_val.clone())
                {
                    return Err(RuntimeError {
                        token: identifier.clone(),
                        message: format!("identifier {identifier} not defined."),
                    });
                }
                Ok(right_val)
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
                    (true, TokenTypes::And) | (false, TokenTypes::Or) => self.evaluate(right),
                    _ => Err(RuntimeError {
                        token: logical.clone(),
                        message: "invalid logical operation.".into(),
                    }),
                }
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                match callee.as_ref() {
                    Expr::Get { object, name } => {
                        if let Some(value) = self.call_get_handle(paren, arguments, object, name) {
                            return value;
                        }
                    }
                    Expr::Super { identifier } => {
                        // 1. Get __super from current env → superclass name
                        let superclass_name = self.environment.borrow().get_cloned("__super");
                        // 2. Get this from current env → current instance
                        let this_class_name = self.environment.borrow().get_cloned("this");
                        if let (Some(LoxValue::String(sc_name)), Some(instance)) =
                            (superclass_name, this_class_name)
                        {
                            match self.lookup_class(&sc_name, &identifier.lexeme) {
                                Ok(LoxValue::Function {
                                    name,
                                    parameters,
                                    body,
                                    ..
                                }) => {
                                    let superclass = self.lookup_superclass_of(&sc_name, &name);
                                    return self.bind_this_method(
                                        arguments,
                                        &parameters,
                                        paren,
                                        &name,
                                        &body,
                                        &instance,
                                        &superclass,
                                    );
                                }
                                _ => {
                                    return Err(RuntimeError {
                                        token: identifier.clone(),
                                        message: "no super available.".into(),
                                    });
                                }
                            }
                            // 3. lookup_class(superclass_name, method_name) → find method
                            // 4. lookup_superclass_of(superclass_name, method_name) → get THAT class's superclass
                            // 5. bind_this_method(arguments, ..., this, &that_superclass)
                        }
                    }
                    _ => {}
                }

                // Regular function call
                let identifier = self.evaluate(callee)?;
                match &identifier {
                    LoxValue::Function {
                        name,
                        parameters,
                        body,
                        env,
                    } => {
                        if arguments.len() != parameters.len() {
                            return Err(RuntimeError {
                                token: paren.clone(),
                                message: format!(
                                    "function {name} expected {} params.",
                                    parameters.len()
                                ),
                            });
                        }
                        let old_env = Rc::clone(&self.environment);
                        let fun_env = Rc::new(RefCell::new(Environment {
                            map: HashMap::new(),
                            parent: Some(Rc::clone(&env)),
                        }));
                        for (pram_name, arg_expr) in parameters.iter().zip(arguments.iter()) {
                            let value = self.evaluate(arg_expr)?;
                            fun_env.borrow_mut().map.insert(pram_name.clone(), value);
                        }
                        self.environment = fun_env;
                        match self.execute(&body) {
                            Ok(result) => {
                                self.environment = old_env;
                                Ok(result.unwrap_or(LoxValue::Nil))
                            }
                            Err(err) => {
                                self.environment = old_env;
                                Err(err)
                            }
                        }
                    }
                    LoxValue::Class { name, .. } => Ok(LoxValue::Instance {
                        fields: Rc::new(RefCell::new(HashMap::new())),
                        class_name: name.clone(),
                    }),
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
                env: Rc::clone(&self.environment),
            }),
            Expr::Set {
                object,
                name,
                value,
            } => match object.as_ref() {
                Expr::Literal { identifier } => {
                    let value = self.evaluate(value)?;
                    if self
                        .environment
                        .borrow_mut()
                        .set_field(&identifier.lexeme, name, value)
                    {
                        Ok(LoxValue::Nil)
                    } else {
                        Err(RuntimeError {
                            token: identifier.clone(),
                            message: "only instances have fields.".into(),
                        })
                    }
                }

                _ => Err(RuntimeError {
                    token: Token {
                        token_type: TokenTypes::Identifier,
                        lexeme: "object".into(),
                        line: 0,
                        column: 0,
                    },
                    message: "invalid left-hand side of assignment.".into(),
                }),
            },
            Expr::Get { object, name } => {
                let obj_val = self.evaluate(object)?;
                match obj_val {
                    LoxValue::Instance { fields, class_name } => {
                        if let Some(field) = fields.borrow().get(name) {
                            Ok(field.clone())
                        } else {
                            self.lookup_class(&class_name, name)
                        }
                    }
                    _ => Err(RuntimeError {
                        token: Token {
                            token_type: TokenTypes::Identifier,
                            lexeme: name.clone(),
                            line: 0,
                            column: 0,
                        },
                        message: "only instances have properties.".into(),
                    }),
                }
            }
            Expr::Super { identifier } => Err(RuntimeError {
                token: identifier.clone(),
                message: "super must be called directly.".into(),
            }),
        }
    }

    fn call_get_handle(
        &mut self,
        paren: &Token,
        arguments: &Vec<Expr>,
        object: &Box<Expr>,
        name: &String,
    ) -> Option<Result<LoxValue, RuntimeError>> {
        if let Expr::Literal {
            identifier: obj_token,
        } = object.as_ref()
        {
            if obj_token.token_type == TokenTypes::Identifier {
                // Evaluate the object to get the instance
                if let Ok(obj_val) = self.evaluate(object) {
                    if let LoxValue::Instance { ref class_name, .. } = obj_val {
                        // Look up the class (clone to avoid borrow issues)
                        let class_opt = self.environment.borrow().get_cloned(&class_name);
                        if let Some(LoxValue::Class {
                            methods,
                            superclass,
                            ..
                        }) = class_opt
                        {
                            match methods.get(name) {
                                Some(LoxValue::Function {
                                    parameters, body, ..
                                }) => {
                                    return Some(self.bind_this_method(
                                        arguments,
                                        parameters,
                                        paren,
                                        name,
                                        body,
                                        &obj_val,
                                        &superclass,
                                    ));
                                }
                                None => {
                                    if let Some(sc_name) = superclass {
                                        if let Ok(LoxValue::Function {
                                            name,
                                            parameters,
                                            body,
                                            ..
                                        }) = self.lookup_class(&sc_name.lexeme, name)
                                        {
                                            let superclass =
                                                self.lookup_superclass_of(&sc_name.lexeme, &name);
                                            return Some(self.bind_this_method(
                                                arguments,
                                                &parameters,
                                                paren,
                                                &name,
                                                &body,
                                                &obj_val,
                                                &superclass,
                                            ));
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        None
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
                    match self.execute(stmt) {
                        Ok(val) => {
                            out = val;
                            if out.is_some() {
                                break;
                            }
                        }
                        Err(err) => {
                            self.environment = old_env;
                            return Err(err);
                        }
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
                        env: Rc::clone(&self.environment),
                    },
                );
                Ok(None)
            }
            Stmt::Class {
                name,
                methods,
                superclass,
            } => {
                let mut class_methods = HashMap::new();
                for (_, stmt) in methods {
                    if let Stmt::Function { name, params, body } = stmt.as_ref() {
                        class_methods.insert(
                            name.clone(),
                            LoxValue::Function {
                                name: name.clone(),
                                parameters: params.clone(),
                                body: body.clone(),
                                env: Rc::clone(&self.environment),
                            },
                        );
                    }
                }

                match superclass {
                    Some(sc) => match self.environment.borrow().get_cloned(&sc.lexeme) {
                        Some(LoxValue::Class { .. }) => {}
                        Some(_) => {
                            return Err(RuntimeError {
                                token: sc.clone(),
                                message: "class is not a Class.".into(),
                            });
                        }
                        None => {
                            return Err(RuntimeError {
                                token: sc.clone(),
                                message: "class not found.".into(),
                            });
                        }
                    },
                    None => {}
                }
                self.environment.borrow_mut().map.insert(
                    name.clone(),
                    LoxValue::Class {
                        name: name.clone(),
                        methods: class_methods,
                        superclass: superclass.clone(),
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
        }
    }
    fn lookup_superclass_of(&self, class_name: &str, method_name: &str) -> Option<Token> {
        match self.environment.borrow().get_cloned(&class_name) {
            Some(LoxValue::Class {
                methods,
                superclass,
                ..
            }) => match (methods.get(method_name), &superclass) {
                (Some(_), _) => superclass,
                (None, Some(sc)) => self.lookup_superclass_of(&sc.lexeme, method_name),
                _ => None,
            },
            _ => None,
        }
    }

    fn lookup_class(&self, class_name: &str, method_name: &str) -> Result<LoxValue, RuntimeError> {
        match self.environment.borrow().get_cloned(&class_name) {
            Some(LoxValue::Class {
                methods,
                superclass,
                ..
            }) => match (methods.get(method_name), superclass) {
                (Some(method), _) => Ok(method.clone()),
                (None, Some(sc)) => self.lookup_class(&sc.lexeme, method_name),
                (None, None) => Err(RuntimeError {
                    token: Token {
                        token_type: TokenTypes::Identifier,
                        lexeme: method_name.into(),
                        line: 0,
                        column: 0,
                    },
                    message: "undefined property.".into(),
                }),
            },
            Some(_) => Err(RuntimeError {
                token: Token {
                    token_type: TokenTypes::Identifier,
                    lexeme: class_name.into(),
                    line: 0,
                    column: 0,
                },
                message: "class is not a Class.".into(),
            }),
            None => Err(RuntimeError {
                token: Token {
                    token_type: TokenTypes::Identifier,
                    lexeme: class_name.into(),
                    line: 0,
                    column: 0,
                },
                message: "class not found.".into(),
            }),
        }
    }

    fn bind_this_method(
        &mut self,
        arguments: &Vec<Expr>,
        parameters: &Vec<String>,
        paren: &Token,
        name: &str,
        body: &Stmt,
        obj_val: &LoxValue,
        superclass: &Option<Token>,
    ) -> Result<LoxValue, RuntimeError> {
        if arguments.len() != parameters.len() {
            return Err(RuntimeError {
                token: paren.clone(),
                message: format!("method {name} expected {} params.", parameters.len()),
            });
        }
        let old_env = Rc::clone(&self.environment);
        let fun_env = Rc::new(RefCell::new(Environment {
            map: HashMap::new(),
            parent: Some(Rc::clone(&self.environment)),
        }));
        // Bind 'this' to the actual instance
        fun_env
            .borrow_mut()
            .map
            .insert("this".into(), obj_val.clone());
        if let Some(sc_token) = superclass {
            fun_env
                .borrow_mut()
                .map
                .insert("__super".into(), LoxValue::String(sc_token.lexeme.clone()));
        }
        // Bind parameters
        for (pram_name, arg_expr) in parameters.iter().zip(arguments.iter()) {
            let value = self.evaluate(arg_expr)?;
            fun_env.borrow_mut().map.insert(pram_name.clone(), value);
        }
        self.environment = fun_env;
        match self.execute(&body) {
            Ok(result) => {
                self.environment = old_env;
                Ok(result.unwrap_or(LoxValue::Nil))
            }
            Err(err) => {
                self.environment = old_env;
                Err(err)
            }
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
            TokenTypes::EqualEqual | TokenTypes::BangEqual => self.equality_eval(left, right, &op),
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
        let token_type = &op.token_type;
        let should_flip = match token_type {
            TokenTypes::EqualEqual => false,
            TokenTypes::BangEqual => true,
            _ => {
                return Err(RuntimeError {
                    token: op.clone(),
                    message: "expected `!=` or `==`.".into(),
                });
            }
        };
        if let Some(v) = self.values_equal(left, right) {
            if should_flip {
                Ok(LoxValue::Bool(!v))
            } else {
                Ok(LoxValue::Bool(v))
            }
        } else {
            Err(RuntimeError {
                token: op.clone(),
                message: "wrong call/type to equality.".into(),
            })
        }
    }

    fn values_equal(&self, left: &LoxValue, right: &LoxValue) -> Option<bool> {
        match (left, right) {
            (LoxValue::Bool(l), LoxValue::Bool(r)) => Some(l == r),
            (LoxValue::Number(l), LoxValue::Number(r)) => Some(l == r),
            (LoxValue::String(l), LoxValue::String(r)) => Some(l == r),
            (LoxValue::Nil, LoxValue::Nil) => Some(true),
            _ => None,
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
                let zero = f64::from(0);
                if b == &zero {
                    Err(RuntimeError {
                        token: op.clone(),
                        message: "division by 0.0 is not valid.".into(),
                    })
                } else {
                    Ok(LoxValue::Number(a / b))
                }
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

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, scanner::Scanner};

    use super::*;

    #[test]
    fn inter_var() {
        let mut scanner = Scanner {
            source_code: "var x = 2;".into(),
            line: 1,
            column: 0,
        };
        let expected = None;
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse_program().unwrap();
        let mut inter = Interpreter::new();
        for stmt in statements {
            let result = inter.execute(&stmt).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn inter_fun_capture() {
        let mut scanner = Scanner {
            source_code: "fun makeCounter() {
                            var count = 0;
                            fun counter() {
                              count = count + 1;
                              return count;
                            }
                            return counter;
                          }

                          var c = makeCounter();
                          print c();  // should print 1
                          print c();  // should print 2"
                .into(),
            line: 1,
            column: 0,
        };
        let expected = vec![None, None, None, None];
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse_program().unwrap();
        let mut inter = Interpreter::new();
        for (stmt, exp) in statements.iter().zip(expected.iter()) {
            assert_eq!(inter.execute(&stmt).unwrap(), *exp);
        }
    }

    // Helper to run a Lox program and return all execute results in order.
    fn run_program(source: &str) -> Vec<Option<LoxValue>> {
        let mut scanner = Scanner {
            source_code: source.into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse_program().unwrap();
        let mut inter = Interpreter::new();
        let mut results = Vec::new();
        for stmt in &statements {
            results.push(inter.execute(stmt).unwrap());
        }
        results
    }

    // Helper that just verifies a program executes without error.
    fn assert_program_ok(source: &str) {
        run_program(source);
    }

    #[test]
    fn inter_arithmetic() {
        assert_program_ok("print 2 + 3 * 4;");
    }

    #[test]
    fn inter_string_concat() {
        assert_program_ok(r#"print "hello" + " world";"#);
    }

    #[test]
    fn inter_comparison() {
        assert_program_ok("print 5 > 3; print 3 > 5;");
    }

    #[test]
    fn inter_equality() {
        assert_program_ok("print nil == nil; print 1 == 2;");
    }

    #[test]
    fn inter_unary_minus() {
        assert_program_ok("print -5;");
    }

    #[test]
    fn inter_unary_bang() {
        assert_program_ok("print !true; print !false;");
    }

    #[test]
    fn inter_variable_decl_and_use() {
        assert_program_ok("var x = 10; print x;");
    }

    #[test]
    fn inter_variable_assign() {
        assert_program_ok("var x = 1; x = 2; print x;");
    }

    #[test]
    fn inter_if_true() {
        assert_program_ok("if (true) print 1;");
    }

    #[test]
    fn inter_if_else() {
        assert_program_ok("if (false) print 1; else print 2;");
    }

    #[test]
    fn inter_while_loop() {
        assert_program_ok("var i = 0; while (i < 3) { print i; i = i + 1; }");
    }

    #[test]
    fn inter_for_loop() {
        assert_program_ok("for (var i = 0; i < 3; i = i + 1) { print i; }");
    }

    #[test]
    fn inter_function_call() {
        assert_program_ok("fun add(a, b) { return a + b; } print add(1, 2);");
    }

    #[test]
    fn inter_function_no_return() {
        assert_program_ok("fun noop() { print 1; } noop();");
    }

    #[test]
    fn inter_return_value() {
        assert_program_ok("fun f() { return 42; } print f();");
    }

    #[test]
    fn inter_logical_and() {
        assert_program_ok("print true and false;");
    }

    #[test]
    fn inter_logical_or() {
        assert_program_ok("print false or true;");
    }

    #[test]
    fn inter_block_scope() {
        assert_program_ok("var x = 1; { var x = 2; print x; } print x;");
    }

    #[test]
    fn inter_lambda() {
        assert_program_ok("var f = fun (x) { return x + 1; }; print f(5);");
    }

    #[test]
    fn inter_nested_function() {
        assert_program_ok(
            "fun outer() { fun inner() { return 1; } return inner(); }
            print outer();",
        );
    }

    #[test]
    fn class_creation() {
        assert_program_ok("class name { fun method() { return 12; } }");
    }

    #[test]
    fn class_object() {
        assert_program_ok(
            "class name { fun method() { return 12; } }
            var b = name();",
        );
    }

    #[test]
    fn class_object_function_call() {
        assert_program_ok(
            "class name { fun method() { return 12; } }
            var b = name(); b.method();",
        );
    }

    #[test]
    fn class_object_set_member() {
        assert_program_ok(
            "class name { fun method() { return 12; } }
            var b = name(); b.x = 12;",
        );
    }

    #[test]
    fn class_object_get_member() {
        assert_program_ok(
            "class name { fun method() { return 12; } }
            var b = name(); b.x = 12; var abc = b.x;",
        );
    }

    #[test]
    fn class_method_with_params() {
        assert_program_ok(
            "class Calc { fun add(a, b) { return a + b; } }
            var c = Calc(); print c.add(5, 3);",
        );
    }

    #[test]
    fn class_multiple_methods() {
        assert_program_ok(
            "class Multi { fun one() { return 1; } fun two() { return 2; } }
            var m = Multi();
            print m.one();
            print m.two();",
        );
    }

    #[test]
    fn class_multiple_instances() {
        assert_program_ok(
            "class Box { }
            var a = Box();
            var b = Box();
            a.x = 1; b.x = 2; 
            print a.x; print b.x;",
        );
    }

    #[test]
    fn class_empty() {
        assert_program_ok(
            "class Empty { }
            var e = Empty();",
        );
    }

    #[test]
    fn class_with_this() {
        assert_program_ok(
            " class Box {
                fun set(value) {
                    this.value = value;     // set a field on the instance
                }
                fun get() {
                    return this.value;       // read a field from the instance
                }
            }
            var b = Box();
            b.set(42);
            print b.get();",
        );
    }

    #[test]
    fn class_this_multiple_instances_isolated() {
        assert_program_ok(
            "class Box {
                fun set(value) {
                    this.value = value;
                }
                fun get() {
                    return this.value;
                }
            }
            var a = Box();
            var b = Box();
            a.set(10);
            b.set(20);
            print a.get();
            print b.get();",
        );
    }

    #[test]
    fn class_this_modifies_field_multiple_times() {
        assert_program_ok(
            "class Counter {
                fun inc() {
                    this.count = this.count + 1;
                    return this.count;
                }
            }
            var c = Counter();
            c.count = 0;
            print c.inc();
            print c.inc();
            print c.inc();",
        );
    }

    #[test]
    fn class_this_calls_another_method() {
        assert_program_ok(
            "class Greeter {
                fun greet(name) {
                    return this.format(name);
                }
                fun format(name) {
                    return name;
                }
            }
            var g = Greeter();
            print g.greet(\"hello\");",
        );
    }

    #[test]
    fn class_field_set_directly_read_via_this() {
        assert_program_ok(
            "class Box {
                fun get() {
                    return this.value;
                }
            }
            var b = Box();
            b.value = 42;
            print b.get();",
        );
    }

    #[test]
    fn class_with_super() {
        assert_program_ok(
            "class Shape {
                fun draw() {
                    print \"draw\";
                }
            }
            class Box < Shape {
                fun get() {
                    return this.value;
                }
            }
            var b = Box();
            b.value = 42;
            print b.get();
            b.draw();",
        );
    }

    #[test]
    fn class_with_super_super() {
        assert_program_ok(
            "class Object {
                fun location() {
                    return 0.0;
                }
            }
            class Shape < Object {
                fun draw() {
                    print \"draw\";
                }
            }
            class Box < Shape {
                fun get() {
                    return this.value;
                }
            }
            var b = Box();
            b.value = 42;
            print b.get();
            b.draw();
            print b.location();",
        );
    }

    // --- Negative (error) tests ---

    fn assert_runtime_error(source: &str, expected_substring: &str) {
        let mut scanner = Scanner {
            source_code: source.into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse_program().unwrap();
        let mut inter = Interpreter::new();
        for stmt in &statements {
            let result = inter.execute(stmt);
            if result.is_err() {
                let err = result.unwrap_err();
                assert!(
                    err.message.contains(expected_substring),
                    "error message '{}' did not contain '{}'",
                    err.message,
                    expected_substring
                );
                return;
            }
        }
        panic!(
            "expected runtime error containing '{}' but program completed successfully",
            expected_substring
        );
    }

    #[test]
    fn runtime_error_undefined_variable() {
        assert_runtime_error("print x;", "not found");
    }

    #[test]
    fn runtime_error_assign_undefined() {
        assert_runtime_error("x = 5;", "not defined");
    }

    #[test]
    fn runtime_error_add_string_number() {
        assert_runtime_error(r#"print 1 + "hello";"#, "wrong call/type to arithmetic.");
    }

    #[test]
    fn runtime_error_divide_by_zero() {
        assert_runtime_error("print 10 / 0;", "division by");
    }

    #[test]
    fn runtime_error_negate_non_number() {
        assert_runtime_error(
            r#"print -"hello";"#,
            "expected right side to be number or bool.",
        );
    }

    #[test]
    fn runtime_error_compare_string() {
        assert_runtime_error(r#"print "a" < "b";"#, "unexpected call for comparison");
    }

    #[test]
    fn runtime_error_call_non_function() {
        assert_runtime_error("var x = 5; print x();", "expected function.");
    }

    #[test]
    fn runtime_error_wrong_arity() {
        assert_runtime_error(
            "fun f(a) { return a; } print f(1, 2);",
            "expected 1 params.",
        );
    }

    #[test]
    fn runtime_error_equality_incompatible() {
        assert_runtime_error(r#"print 1 == "hello";"#, "wrong call/type to equality.");
    }

    #[test]
    fn runtime_error_arithmetic_non_number() {
        assert_runtime_error("print true + 1;", "wrong call/type to arithmetic.");
    }

    #[test]
    fn runtime_error_undefined_property() {
        assert_runtime_error(
            "class Box { } var b = Box(); print b.nope;",
            "undefined property",
        );
    }

    #[test]
    fn runtime_error_property_on_non_instance() {
        assert_runtime_error("var x = 5; print x.y;", "only instances have properties");
    }

    #[test]
    fn runtime_error_set_on_non_instance() {
        assert_runtime_error("var x = 5; x.y = 10;", "only instances have fields");
    }

    #[test]
    fn runtime_error_call_field() {
        assert_runtime_error(
            "class Box { } var b = Box(); b.f = 5; b.f();",
            "expected function",
        );
    }

    #[test]
    fn runtime_error_this_outside_class() {
        assert_runtime_error("print this;", "this");
    }

    #[test]
    fn runtime_error_this_in_standalone_function() {
        assert_runtime_error("fun f() { return this; } print f();", "this");
    }

    #[test]
    fn runtime_error_call_undefined_method() {
        assert_runtime_error(
            "class Box { } var b = Box(); b.nonexistent();",
            "undefined property",
        );
    }

    #[test]
    fn runtime_error_this_access_undefined_field() {
        assert_runtime_error(
            "class Box {
                fun get() {
                    return this.neverSet;
                }
            }
            var b = Box();
            print b.get();",
            "undefined property",
        );
    }

    #[test]
    fn class_inherit_and_override() {
        assert_program_ok(
            "class Animal {
                fun speak() {
                    return \"generic\";
                }
            }
            class Dog < Animal {
                fun speak() {
                    return \"woof\";
                }
            }
            var a = Animal();
            var d = Dog();
            print a.speak();
            print d.speak();",
        );
    }

    #[test]
    fn class_inherit_method_with_this() {
        assert_program_ok(
            "class Shape {
                fun describe() {
                    return this.name;
                }
            }
            class Box < Shape {}
            var b = Box();
            b.name = \"myBox\";
            print b.describe();",
        );
    }

    #[test]
    fn class_inherit_multiple_levels_field_access() {
        assert_program_ok(
            "class A {
                fun getA() { return this.val; }
            }
            class B < A {
                fun getB() { return this.val; }
            }
            class C < B {
                fun getC() { return this.val; }
            }
            var c = C();
            c.val = 99;
            print c.getA();
            print c.getB();
            print c.getC();",
        );
    }

    #[test]
    fn class_subclass_has_own_methods_and_inherited() {
        assert_program_ok(
            "class Base {
                fun baseMethod() { return 1; }
            }
            class Sub < Base {
                fun subMethod() { return 2; }
            }
            var s = Sub();
            print s.baseMethod();
            print s.subMethod();",
        );
    }

    #[test]
    fn class_inherit_set_and_get_across_chain() {
        assert_program_ok(
            "class A {}
            class B < A {}
            class C < B {}
            var c = C();
            c.x = 10;
            c.y = 20;
            print c.x;
            print c.y;",
        );
    }

    #[test]
    fn class_inherit_get_super() {
        assert_program_ok(
            "class A {
                fun a_class(){
                    return 0;
                }
                fun set_a(param){
                    this.a = param;
                }
            }
            class B < A {
                fun b_class(){
                    return super.a_class();
                }
            }
            class C < B {
                fun c_class(){return super.b_class();}
            }
            var c = C();
            c.x = 10;
            c.y = 20;
            c.c_class();
            print c.x;
            print c.y;",
        );
    }

    #[test]
    fn super_with_parameters() {
        assert_program_ok(
            "class A {
                fun greet(msg) { return msg; }
            }
            class B < A {
                fun greet(msg) { return super.greet(msg); }
            }
            var b = B();
            print b.greet(\"hello\");",
        );
    }

    #[test]
    fn super_modifies_this_fields() {
        assert_program_ok(
            "class A {
                fun set_name() { this.name = \"from_super\"; }
            }
            class B < A {
                fun set_name() { super.set_name(); }
            }
            var b = B();
            b.set_name();
            print b.name;",
        );
    }

    #[test]
    fn super_multi_level_chain() {
        assert_program_ok(
            "class A { fun who() { return \"A\"; } }
            class B < A { fun who() { return \"B\" + super.who(); } }
            class C < B { fun who() { return \"C\" + super.who(); } }
            var c = C();
            print c.who();",
        );
    }

    #[test]
    fn super_return_value_used() {
        assert_program_ok(
            "class A {
                fun val() { return 42; }
            }
            class B < A {
                fun val() {
                    var x = super.val();
                    print x;
                    return x;
                }
            }
            var b = B();
            print b.val();",
        );
    }

    // --- Negative: inheritance errors ---

    #[test]
    fn runtime_error_super_outside_class() {
        assert_runtime_error("super.method();", "super must be called");
    }

    #[test]
    fn runtime_error_super_undefined_method() {
        assert_runtime_error(
            "class A { }
            class B < A { fun m() { super.nonexistent(); } }
            var b = B();
            b.m();",
            "no super available",
        );
    }

    #[test]
    fn runtime_error_super_no_superclass() {
        assert_runtime_error(
            "class A { fun m() { super.foo(); } }
            var a = A();
            a.m();",
            "super must be called",
        );
    }

    #[test]
    fn runtime_error_call_undefined_method_on_subclass() {
        assert_runtime_error(
            "class Base { fun baseMethod() { return 1; } }
            class Sub < Base {}
            var s = Sub();
            s.nonexistent();",
            "undefined property",
        );
    }

    #[test]
    fn runtime_error_superclass_not_a_class() {
        assert_runtime_error(
            "var x = 5;
            class Sub < x {}",
            "class is not a Class",
        );
    }

    #[test]
    fn runtime_error_undefined_superclass() {
        assert_runtime_error("class Sub < NotFound {}", "class not found");
    }
}
