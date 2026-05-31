use std::collections::HashMap;

use crate::scanner::{Expr, Stmt, Token, TokenTypes};

#[derive(Debug)]
pub struct ParserError {
    pub token: Token,
    pub message: String,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
    fn advance(&mut self) -> Token {
        let token = self.tokens[self.current].clone();
        self.current += 1;
        token
    }
    fn check(&self, token_type: TokenTypes) -> bool {
        if let Some(token) = self.peek()
            && &token.token_type == &token_type
        {
            return true;
        }
        false
    }
    fn check_and_advance(&mut self, token_type: TokenTypes) -> bool {
        if self.check(token_type) {
            let _ = self.advance();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, token_type: TokenTypes) -> Result<Token, ParserError> {
        let error_msg = format!("expected {:?}.", &token_type);
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            let found = self.peek().cloned().unwrap(); // always found (EOF max)
            Err(ParserError {
                token: found,
                message: error_msg,
            })
        }
    }

    pub fn assignment(&mut self) -> Result<Expr, ParserError> {
        let left = self.or()?;
        if self.check(TokenTypes::Equal) {
            match left {
                Expr::Literal { identifier } => {
                    let _ = self.advance();
                    let right = self.assignment()?;
                    return Ok(Expr::Assign {
                        identifier: identifier,
                        right: Box::new(right),
                    });
                }
                Expr::Get { object, name } => {
                    let _ = self.advance();
                    let right = self.assignment()?;
                    return Ok(Expr::Set {
                        object,
                        name,
                        value: Box::new(right),
                    });
                }
                _ => {
                    return Err(ParserError {
                        token: self.peek().cloned().unwrap(),
                        message: "unexpected token.".to_string(),
                    });
                }
            }
        }
        Ok(left)
    }
    fn or(&mut self) -> Result<Expr, ParserError> {
        let left = self.and()?;
        if self.check(TokenTypes::Or) {
            let logical = self.advance();
            let right = self.or()?;
            return Ok(Expr::Logical {
                left: Box::new(left),
                logical: logical,
                right: Box::new(right),
            });
        }
        Ok(left)
    }
    fn and(&mut self) -> Result<Expr, ParserError> {
        let left = self.equality()?;
        if self.check(TokenTypes::And) {
            let logical = self.advance();
            let right = self.and()?;
            return Ok(Expr::Logical {
                left: Box::new(left),
                logical: logical,
                right: Box::new(right),
            });
        }
        Ok(left)
    }
    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.comparison()?;
        while self.check(TokenTypes::EqualEqual) || self.check(TokenTypes::BangEqual) {
            let operation = self.advance();
            let right = self.comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                operation,
                right: Box::new(right),
            }
        }
        Ok(left)
    }
    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.term()?;
        while self.check(TokenTypes::Less)
            || self.check(TokenTypes::LessEqual)
            || self.check(TokenTypes::Greater)
            || self.check(TokenTypes::GreaterEqual)
        {
            let operation = self.advance();
            let right = self.term()?;
            left = Expr::Binary {
                left: Box::new(left),
                operation,
                right: Box::new(right),
            }
        }
        Ok(left)
    }
    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.factor()?;
        while self.check(TokenTypes::Plus) || self.check(TokenTypes::Minus) {
            let operation = self.advance();
            let right = self.factor()?;
            left = Expr::Binary {
                left: Box::new(left),
                operation,
                right: Box::new(right),
            }
        }
        Ok(left)
    }
    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.unary()?;
        while self.check(TokenTypes::Star) || self.check(TokenTypes::Slash) {
            let operation = self.advance();
            let right = self.unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                operation,
                right: Box::new(right),
            }
        }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.check(TokenTypes::Minus) || self.check(TokenTypes::Bang) {
            let operation = self.advance();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operation: operation,
                right: Box::new(right),
            });
        }
        self.call()
    }
    fn call(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.primary()?;
        while self.check(TokenTypes::LeftParen) && !self.check(TokenTypes::Eof) {
            if self.check_and_advance(TokenTypes::LeftParen) {
                let paren = self.tokens[self.current - 1].clone();
                let mut arguments = vec![];
                while !self.check(TokenTypes::RightParen) && !self.check(TokenTypes::Eof) {
                    let argument = self.assignment()?;
                    arguments.push(argument);
                    self.check_and_advance(TokenTypes::Comma);
                }
                if self.check_and_advance(TokenTypes::RightParen) {
                    left = Expr::Call {
                        callee: Box::new(left),
                        paren: paren,
                        arguments,
                    };
                } else {
                    let found = self.peek().cloned().unwrap();
                    return Err(ParserError {
                        token: found,
                        message: "expected '}' .".to_string(),
                    });
                }
            }
        }
        Ok(left)
    }
    fn primary(&mut self) -> Result<Expr, ParserError> {
        if self.check(TokenTypes::Number) || self.check(TokenTypes::Identifier) {
            let literal = self.advance();
            if self.check(TokenTypes::Dot) {
                let _ = self.advance();
                let member = self.expect(TokenTypes::Identifier)?;
                return Ok(Expr::Get {
                    object: Box::new(Expr::Literal {
                        identifier: literal,
                    }),
                    name: member.lexeme,
                });
            }
            return Ok(Expr::Literal {
                identifier: literal,
            });
        } else if self.check(TokenTypes::LeftParen) {
            let open_group = self.advance();
            let right = self.equality()?;
            if self.check(TokenTypes::RightParen) {
                let _ = self.advance();
                return Ok(Expr::Grouping {
                    expression: Box::new(right),
                });
            } else {
                return Err(ParserError {
                    token: self.peek().cloned().unwrap(),
                    message: format!("expected closing for {}", open_group.lexeme),
                });
            }
        } else if self.check(TokenTypes::True)
            || self.check(TokenTypes::False)
            || self.check(TokenTypes::Nil)
            || self.check(TokenTypes::This)
        {
            let key_word = self.advance();
            // Handle this.property like object.property
            if self.check(TokenTypes::Dot) && key_word.token_type == TokenTypes::This {
                let _ = self.advance();
                let member = self.expect(TokenTypes::Identifier)?;
                return Ok(Expr::Get {
                    object: Box::new(Expr::Literal {
                        identifier: key_word,
                    }),
                    name: member.lexeme,
                });
            }
            return Ok(Expr::Literal {
                identifier: key_word,
            });
        } else if self.check(TokenTypes::String) {
            let string = self.advance();
            return Ok(Expr::Literal { identifier: string });
        } else if self.check(TokenTypes::Fun) {
            return self.parse_function_expr();
        }
        return Err(ParserError {
            token: self.peek().cloned().unwrap(),
            message: "expected primary.".to_string(),
        });
    }
    fn parse_function_expr(&mut self) -> Result<Expr, ParserError> {
        let left = self.advance();
        if self.check_and_advance(TokenTypes::LeftParen) {
            let mut arguments = vec![];
            while !self.check(TokenTypes::RightParen) && !self.check(TokenTypes::Eof) {
                arguments.push(self.advance().lexeme);
                self.check_and_advance(TokenTypes::Comma);
            }
            self.expect(TokenTypes::RightParen)?;
            return Ok(Expr::Lambda {
                params: arguments,
                body: Box::new(self.parse_block()?),
            });
        }
        Ok(Expr::Literal { identifier: left })
    }

    fn check_semicolon(&mut self, statement: Stmt) -> Result<Stmt, ParserError> {
        if self.check(TokenTypes::Semicolon) {
            let _ = self.advance();
            return Ok(statement);
        }
        let found = self.peek().cloned().unwrap();
        Err(ParserError {
            token: found,
            message: "expected ';'.".to_string(),
        })
    }
    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut v = Vec::new();
        if self.tokens.is_empty() {
            return Err(ParserError {
                token: Token {
                    token_type: TokenTypes::Nil,
                    lexeme: "nil".into(),
                    line: 0,
                    column: 0,
                },
                message: "no token exist.".into(),
            });
        }
        while !self.check(TokenTypes::Eof) {
            v.push(self.parse_statement()?);
        }
        Ok(v)
    }
    pub fn parse_statement(&mut self) -> Result<Stmt, ParserError> {
        if self.check_and_advance(TokenTypes::Print) {
            return self.parse_print();
        } else if self.check_and_advance(TokenTypes::Var) {
            return self.parse_var();
        } else if self.check_and_advance(TokenTypes::If) {
            return self.parse_if();
        } else if self.check_and_advance(TokenTypes::While) {
            return self.parse_while();
        } else if self.check_and_advance(TokenTypes::For) {
            return self.parse_for();
        } else if self.check(TokenTypes::LeftBrace) {
            return self.parse_block();
        } else if self.check_and_advance(TokenTypes::Return) {
            return self.parse_return();
        } else if self.check_and_advance(TokenTypes::Fun) {
            return self.parse_function();
        } else if self.check_and_advance(TokenTypes::Class) {
            return self.parse_class();
        }
        return self.parse_expr();
    }
    fn parse_expr(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.assignment()?;
        return self.check_semicolon(Stmt::Expression { expr });
    }
    fn parse_var(&mut self) -> Result<Stmt, ParserError> {
        let name = self.expect(TokenTypes::Identifier)?.lexeme;
        let mut value = None;
        if self.check_and_advance(TokenTypes::Equal) {
            value = Some(self.assignment()?);
        }
        return self.check_semicolon(Stmt::Var { name, value });
    }
    fn parse_print(&mut self) -> Result<Stmt, ParserError> {
        let print_stmt = self.assignment()?;
        return self.check_semicolon(Stmt::Print { expr: print_stmt });
    }
    fn parse_if(&mut self) -> Result<Stmt, ParserError> {
        self.expect(TokenTypes::LeftParen)?;
        let expr = self.assignment()?;
        self.expect(TokenTypes::RightParen)?;
        let body = self.parse_statement()?;
        let mut else_branch = None;
        if self.check_and_advance(TokenTypes::Else) {
            else_branch = Some(Box::new(self.parse_statement()?));
        }
        Ok(Stmt::If {
            condition: expr,
            body: Box::new(body),
            else_branch: else_branch,
        })
    }
    fn parse_while(&mut self) -> Result<Stmt, ParserError> {
        self.expect(TokenTypes::LeftParen)?;
        let expr = self.assignment()?;
        self.expect(TokenTypes::RightParen)?;
        let body = self.parse_statement()?;
        Ok(Stmt::While {
            condition: expr,
            body: Box::new(body),
        })
    }
    fn parse_for(&mut self) -> Result<Stmt, ParserError> {
        self.expect(TokenTypes::LeftParen)?;
        let mut data = Vec::new();
        let init = self.parse_statement()?;
        data.push(init);
        let condition = self.parse_statement()?;
        let while_expr;
        match condition {
            Stmt::Expression { expr } => while_expr = expr,
            _ => {
                let found = self.peek().cloned().unwrap();
                return Err(ParserError {
                    token: found,
                    message: "expected an expression.".to_string(),
                });
            }
        }
        let increment = Stmt::Expression {
            expr: self.assignment()?,
        };
        self.expect(TokenTypes::RightParen)?;
        let body = self.parse_statement()?;
        let body_group = Stmt::Block {
            data: vec![body, increment],
        };
        data.push(Stmt::While {
            condition: while_expr,
            body: Box::new(body_group),
        });
        Ok(Stmt::Block { data: data })
    }
    fn parse_block(&mut self) -> Result<Stmt, ParserError> {
        if !self.check_and_advance(TokenTypes::LeftBrace) {
            let found = self.peek().cloned().unwrap();
            return Err(ParserError {
                token: found,
                message: "expected open brace.".to_string(),
            });
        }
        let mut data = vec![];
        while !self.check(TokenTypes::Eof) && !self.check(TokenTypes::RightBrace) {
            data.push(self.parse_statement()?);
        }
        self.expect(TokenTypes::RightBrace)?;
        Ok(Stmt::Block { data })
    }
    fn parse_return(&mut self) -> Result<Stmt, ParserError> {
        if self.check_and_advance(TokenTypes::Semicolon) {
            return Ok(Stmt::Return { value: None });
        }
        let return_value = self.assignment()?;
        return self.check_semicolon(Stmt::Return {
            value: Some(return_value),
        });
    }
    fn parse_function(&mut self) -> Result<Stmt, ParserError> {
        let name = self.expect(TokenTypes::Identifier)?.lexeme;
        self.expect(TokenTypes::LeftParen)?;
        let mut params = vec![];
        while !self.check(TokenTypes::RightParen) && !self.check(TokenTypes::Eof) {
            if self.check(TokenTypes::Identifier) {
                params.push(self.advance().lexeme);
                self.check_and_advance(TokenTypes::Comma);
            } else {
                return Err(ParserError {
                    token: self.peek().cloned().unwrap(),
                    message: "expected param name.".to_string(),
                });
            }
        }
        self.expect(TokenTypes::RightParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::Function {
            name: name,
            params: params,
            body: Box::new(body),
        })
    }

    fn parse_class(&mut self) -> Result<Stmt, ParserError> {
        let name = self.expect(TokenTypes::Identifier)?.lexeme;
        self.expect(TokenTypes::LeftBrace)?;
        let mut methods = HashMap::new();
        while !self.check(TokenTypes::RightBrace) && !self.check(TokenTypes::Eof) {
            let stmt = self.parse_statement()?;
            match &stmt {
                Stmt::Function { name, .. } => {
                    methods.insert(name.clone(), Box::new(stmt));
                }
                _ => {
                    return Err(ParserError {
                        token: self.peek().cloned().unwrap(),
                        message: "invalid token in class, expected member/functions.".into(),
                    });
                }
            };
        }
        self.expect(TokenTypes::RightBrace)?;
        Ok(Stmt::Class { name, methods })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    // Helper to create a Token for testing
    fn mk_token(token_type: TokenTypes, lexeme: &str, line: usize, column: usize) -> Token {
        Token {
            token_type,
            lexeme: lexeme.into(),
            line,
            column,
        }
    }

    // Helpers for common tokens (assuming line 1, column 0 for simplicity)
    fn lit_id(lexeme: &str, column: usize) -> Expr {
        Expr::Literal {
            identifier: mk_token(TokenTypes::Identifier, lexeme, 1, column),
        }
    }

    fn lit_num(lexeme: &str, column: usize) -> Expr {
        Expr::Literal {
            identifier: mk_token(TokenTypes::Number, lexeme, 1, column),
        }
    }

    fn lit_str(lexeme: &str, column: usize) -> Expr {
        Expr::Literal {
            identifier: mk_token(TokenTypes::String, lexeme, 1, column),
        }
    }

    fn lit_true(column: usize) -> Expr {
        Expr::Literal {
            identifier: mk_token(TokenTypes::True, "true", 1, column),
        }
    }

    fn lit_false(column: usize) -> Expr {
        Expr::Literal {
            identifier: mk_token(TokenTypes::False, "false", 1, column),
        }
    }

    // fn lit_nil(column: usize) -> Expr {
    //     Expr::Literal {
    //         identifier: mk_token(TokenTypes::Nil, "nil", 1, column),
    //     }
    // }

    #[test]
    fn exprs_assign() {
        let mut scanner = Scanner {
            source_code: "x=2".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Assign {
            identifier: mk_token(TokenTypes::Identifier, "x", 1, 1),
            right: Box::new(lit_num("2", 3)),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn exprs_binary() {
        let mut scanner = Scanner {
            source_code: "x==2".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Binary {
            left: Box::new(lit_id("x", 1)),
            operation: Token {
                token_type: TokenTypes::EqualEqual,
                lexeme: "==".into(),
                line: 1,
                column: 2,
            },
            right: Box::new(lit_num("2", 4)),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);

        let mut scanner = Scanner {
            source_code: "3>2".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Binary {
            left: Box::new(lit_num("3", 1)),
            operation: Token {
                token_type: TokenTypes::Greater,
                lexeme: ">".into(),
                line: 1,
                column: 2,
            },
            right: Box::new(lit_num("2", 3)),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_unary() {
        let mut scanner = Scanner {
            source_code: "-5".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Unary {
            operation: Token {
                token_type: TokenTypes::Minus,
                lexeme: "-".into(),
                line: 1,
                column: 1,
            },
            right: Box::new(lit_num("5", 2)),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_grouping() {
        let mut scanner = Scanner {
            source_code: "(1 + 2)".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Grouping {
            expression: Box::new(Expr::Binary {
                left: Box::new(lit_num("1", 2)),
                operation: Token {
                    token_type: TokenTypes::Plus,
                    lexeme: "+".into(),
                    line: 1,
                    column: 4,
                },
                right: Box::new(lit_num("2", 6)),
            }),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_logical_and() {
        let mut scanner = Scanner {
            source_code: "true and false".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Logical {
            left: Box::new(lit_true(1)),
            logical: Token {
                token_type: TokenTypes::And,
                lexeme: "and".into(),
                line: 1,
                column: 6,
            },
            right: Box::new(lit_false(10)),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_logical_or() {
        let mut scanner = Scanner {
            source_code: "true or false".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Logical {
            left: Box::new(lit_true(1)),
            logical: Token {
                token_type: TokenTypes::Or,
                lexeme: "or".into(),
                line: 1,
                column: 6,
            },
            right: Box::new(lit_false(9)),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_precedence() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4) because * binds tighter
        let mut scanner = Scanner {
            source_code: "2 + 3 * 4".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Binary {
            left: Box::new(lit_num("2", 1)),
            operation: Token {
                token_type: TokenTypes::Plus,
                lexeme: "+".into(),
                line: 1,
                column: 3,
            },
            right: Box::new(Expr::Binary {
                left: Box::new(lit_num("3", 5)),
                operation: Token {
                    token_type: TokenTypes::Star,
                    lexeme: "*".into(),
                    line: 1,
                    column: 7,
                },
                right: Box::new(lit_num("4", 9)),
            }),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_call() {
        let mut scanner = Scanner {
            source_code: "foo(1, 2)".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Call {
            callee: Box::new(lit_id("foo", 1)),
            paren: Token {
                token_type: TokenTypes::LeftParen,
                lexeme: "(".into(),
                line: 1,
                column: 4,
            },
            arguments: vec![lit_num("1", 5), lit_num("2", 8)],
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_lambda() {
        let mut scanner = Scanner {
            source_code: "fun (x) { return x; }".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Lambda {
            params: vec!["x".into()],
            body: Box::new(Stmt::Block {
                data: vec![Stmt::Return {
                    value: Some(lit_id("x", 18)),
                }],
            }),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_var_statement() {
        let mut scanner = Scanner {
            source_code: "var x = 5;".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::Var {
            name: "x".into(),
            value: Some(lit_num("5", 9)),
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_var_nil() {
        let mut scanner = Scanner {
            source_code: "var x;".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::Var {
            name: "x".into(),
            value: None,
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_print() {
        let mut scanner = Scanner {
            source_code: "print 42;".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::Print {
            expr: lit_num("42", 7),
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_if() {
        let mut scanner = Scanner {
            source_code: "if (true) print 1;".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::If {
            condition: lit_true(5),
            body: Box::new(Stmt::Print {
                expr: lit_num("1", 17),
            }),
            else_branch: None,
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_if_else() {
        let mut scanner = Scanner {
            source_code: "if (true) print 1; else print 2;".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::If {
            condition: lit_true(5),
            body: Box::new(Stmt::Print {
                expr: lit_num("1", 17),
            }),
            else_branch: Some(Box::new(Stmt::Print {
                expr: lit_num("2", 31),
            })),
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_while() {
        let mut scanner = Scanner {
            source_code: "while (true) print 1;".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::While {
            condition: lit_true(8),
            body: Box::new(Stmt::Print {
                expr: lit_num("1", 20),
            }),
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_block() {
        let mut scanner = Scanner {
            source_code: "{ print 1; print 2; }".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::Block {
            data: vec![
                Stmt::Print {
                    expr: lit_num("1", 9),
                },
                Stmt::Print {
                    expr: lit_num("2", 18),
                },
            ],
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_function() {
        let mut scanner = Scanner {
            source_code: "fun add(a, b) { return a; }".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::Function {
            name: "add".into(),
            params: vec!["a".into(), "b".into()],
            body: Box::new(Stmt::Block {
                data: vec![Stmt::Return {
                    value: Some(lit_id("a", 24)),
                }],
            }),
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_return_value() {
        let mut scanner = Scanner {
            source_code: "return 42;".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::Return {
            value: Some(lit_num("42", 8)),
        }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_return_nil() {
        let mut scanner = Scanner {
            source_code: "return;".into(),
            line: 1,
            column: 0,
        };
        let expected = vec![Stmt::Return { value: None }];

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program().unwrap();
        assert_eq!(stmts, expected);
    }

    #[test]
    fn parse_string_literal() {
        let mut scanner = Scanner {
            source_code: "\"hello\"".into(),
            line: 1,
            column: 0,
        };
        let expected = lit_str("hello", 1);

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_class_set() {
        let mut scanner = Scanner {
            source_code: "x.y = 12".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Set {
            object: Box::new(lit_id("x", 1)),
            name: "y".into(),
            value: Box::new(lit_num("12", 7)),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    #[test]
    fn parse_class_get() {
        let mut scanner = Scanner {
            source_code: "x.y".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Get {
            object: Box::new(lit_id("x", 1)),
            name: "y".into(),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }

    // --- Negative (error) tests ---

    fn parse_expr_from(source: &str) -> Result<Expr, ParserError> {
        let mut scanner = Scanner {
            source_code: source.into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        parser.assignment()
    }

    fn parse_program_from(source: &str) -> Result<Vec<Stmt>, ParserError> {
        let mut scanner = Scanner {
            source_code: source.into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    fn assert_parse_error<T: std::fmt::Debug>(
        result: Result<T, ParserError>,
        expected_substring: &str,
    ) {
        assert!(
            result.is_err(),
            "expected parse error but got: {:?}",
            result
        );
        let err = result.unwrap_err();
        assert!(
            err.message.contains(expected_substring),
            "error message '{}' did not contain '{}'",
            err.message,
            expected_substring
        );
    }

    #[test]
    fn parse_error_missing_semicolon() {
        let result = parse_program_from("print 1");
        assert_parse_error(result, "expected ';'");
    }

    #[test]
    fn parse_error_var_missing_name() {
        let result = parse_program_from("var = 5;");
        assert_parse_error(result, "expected");
    }

    #[test]
    fn parse_error_if_missing_paren() {
        let result = parse_program_from("if true) print 1;");
        assert_parse_error(result, "expected");
    }

    #[test]
    fn parse_error_while_missing_paren() {
        let result = parse_program_from("while true) print 1;");
        assert_parse_error(result, "expected");
    }

    #[test]
    fn parse_error_unclosed_grouping() {
        let result = parse_expr_from("(1 + 2");
        assert_parse_error(result, "expected");
    }

    #[test]
    fn parse_error_unclosed_block() {
        let result = parse_program_from("{ print 1;");
        assert_parse_error(result, "expected");
    }

    #[test]
    fn parse_error_function_missing_name() {
        let result = parse_program_from("fun (a) { return a; }");
        assert_parse_error(result, "expected");
    }

    #[test]
    fn parse_error_call_unclosed_paren() {
        let result = parse_expr_from("foo(1, 2");
        assert_parse_error(result, "expected");
    }

    #[test]
    fn parse_error_for_missing_paren() {
        let result = parse_program_from("for var i = 0; i < 3; i = i + 1) print i;");
        assert_parse_error(result, "expected");
    }
}
