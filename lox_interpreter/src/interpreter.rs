use crate::scanner::{Expr, Stmt, Token, TokenTypes};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
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
    },
    Instance {
        fields: HashMap<String, LoxValue>,
        class_name: String,
    },
}

impl std::fmt::Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoxValue::Bool(v) => {
                if *v {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
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
    fn set_field(&mut self, key: &str, field: &str, value: LoxValue) -> bool {
        match self.map.get_mut(key) {
            Some(v) => match v {
                LoxValue::Instance { fields, .. } => {
                    fields.insert(field.into(), value);
                    true
                }
                _ => false,
            },
            None => false,
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
                        message: format!("identifier {identifier} not defined."),
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
                        return match self.execute(&body) {
                            Ok(result) => {
                                self.environment = old_env;
                                Ok(result.unwrap_or(LoxValue::Nil))
                            }
                            Err(err) => {
                                self.environment = old_env;
                                Err(err)
                            }
                        };
                    }
                    LoxValue::Class { name, .. } => Ok(LoxValue::Instance {
                        fields: HashMap::new(),
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
                        .set_field(identifier, name, value)
                    {
                        Ok(LoxValue::Nil)
                    } else {
                        Err(RuntimeError {
                            token: Token {
                                token_type: TokenTypes::Identifier,
                                lexeme: identifier.into(),
                                line: 0,
                                column: 0,
                            },
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
                        if let Some(field) = fields.get(name) {
                            return Ok(field.clone());
                        }
                        match self.environment.borrow().get_cloned(&class_name) {
                            Some(LoxValue::Class { methods, .. }) => {
                                if let Some(method) = methods.get(name) {
                                    return Ok(method.clone());
                                }
                                Err(RuntimeError {
                                    token: Token {
                                        token_type: TokenTypes::Identifier,
                                        lexeme: name.clone(),
                                        line: 0,
                                        column: 0,
                                    },
                                    message: "undefined property.".into(),
                                })
                            }
                            Some(_) => Err(RuntimeError {
                                token: Token {
                                    token_type: TokenTypes::Identifier,
                                    lexeme: class_name.clone(),
                                    line: 0,
                                    column: 0,
                                },
                                message: "class is not a Class.".into(),
                            }),
                            None => Err(RuntimeError {
                                token: Token {
                                    token_type: TokenTypes::Identifier,
                                    lexeme: class_name.clone(),
                                    line: 0,
                                    column: 0,
                                },
                                message: "class not found.".into(),
                            }),
                        }
                    }
                    _ => Err(RuntimeError {
                        token: Token {
                            token_type: TokenTypes::Identifier,
                            lexeme: "object".into(),
                            line: 0,
                            column: 0,
                        },
                        message: "only instances have properties.".into(),
                    }),
                }
            }
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
            Stmt::Class { name, methods } => {
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
                self.environment.borrow_mut().map.insert(
                    name.clone(),
                    LoxValue::Class {
                        name: name.clone(),
                        methods: class_methods,
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
}
