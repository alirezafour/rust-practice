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
        let error_msg = format!("expected {:?}.", &token_type).to_string();
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
                        identifier,
                        right: Box::new(right),
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
            return Ok(Expr::Literal {
                identifier: literal.lexeme,
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
                    message: format!("expected closing for {}", open_group.lexeme).to_string(),
                });
            }
        } else if self.check(TokenTypes::True)
            || self.check(TokenTypes::False)
            || self.check(TokenTypes::Nil)
        {
            let key_word = self.advance();
            return Ok(Expr::Literal {
                identifier: key_word.lexeme,
            });
        } else if self.check(TokenTypes::String) {
            let string = self.advance();
            return Ok(Expr::Literal {
                identifier: string.lexeme,
            });
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
        Ok(Expr::Literal {
            identifier: left.lexeme,
        })
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
                message: "expected '('.".to_string(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    #[test]
    fn exprs_assign() {
        let mut scanner = Scanner {
            source_code: "x=2".into(),
            line: 1,
            column: 0,
        };
        let expected = Expr::Assign {
            identifier: "x".into(),
            right: Box::new(Expr::Literal {
                identifier: "2".into(),
            }),
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
            left: Box::new(Expr::Literal {
                identifier: "x".into(),
            }),
            operation: Token {
                token_type: TokenTypes::EqualEqual,
                lexeme: "==".into(),
                line: 1,
                column: 2,
            },
            right: Box::new(Expr::Literal {
                identifier: "2".into(),
            }),
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
            left: Box::new(Expr::Literal {
                identifier: "3".into(),
            }),
            operation: Token {
                token_type: TokenTypes::Greater,
                lexeme: ">".into(),
                line: 1,
                column: 2,
            },
            right: Box::new(Expr::Literal {
                identifier: "2".into(),
            }),
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
            right: Box::new(Expr::Literal {
                identifier: "5".into(),
            }),
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
                left: Box::new(Expr::Literal {
                    identifier: "1".into(),
                }),
                operation: Token {
                    token_type: TokenTypes::Plus,
                    lexeme: "+".into(),
                    line: 1,
                    column: 4,
                },
                right: Box::new(Expr::Literal {
                    identifier: "2".into(),
                }),
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
            left: Box::new(Expr::Literal {
                identifier: "true".into(),
            }),
            logical: Token {
                token_type: TokenTypes::And,
                lexeme: "and".into(),
                line: 1,
                column: 6,
            },
            right: Box::new(Expr::Literal {
                identifier: "false".into(),
            }),
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
            left: Box::new(Expr::Literal {
                identifier: "true".into(),
            }),
            logical: Token {
                token_type: TokenTypes::Or,
                lexeme: "or".into(),
                line: 1,
                column: 6,
            },
            right: Box::new(Expr::Literal {
                identifier: "false".into(),
            }),
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
            left: Box::new(Expr::Literal {
                identifier: "2".into(),
            }),
            operation: Token {
                token_type: TokenTypes::Plus,
                lexeme: "+".into(),
                line: 1,
                column: 3,
            },
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal {
                    identifier: "3".into(),
                }),
                operation: Token {
                    token_type: TokenTypes::Star,
                    lexeme: "*".into(),
                    line: 1,
                    column: 7,
                },
                right: Box::new(Expr::Literal {
                    identifier: "4".into(),
                }),
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
            callee: Box::new(Expr::Literal {
                identifier: "foo".into(),
            }),
            paren: Token {
                token_type: TokenTypes::LeftParen,
                lexeme: "(".into(),
                line: 1,
                column: 4,
            },
            arguments: vec![
                Expr::Literal {
                    identifier: "1".into(),
                },
                Expr::Literal {
                    identifier: "2".into(),
                },
            ],
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
                    value: Some(Expr::Literal {
                        identifier: "x".into(),
                    }),
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
            value: Some(Expr::Literal {
                identifier: "5".into(),
            }),
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
            expr: Expr::Literal {
                identifier: "42".into(),
            },
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
            condition: Expr::Literal {
                identifier: "true".into(),
            },
            body: Box::new(Stmt::Print {
                expr: Expr::Literal {
                    identifier: "1".into(),
                },
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
            condition: Expr::Literal {
                identifier: "true".into(),
            },
            body: Box::new(Stmt::Print {
                expr: Expr::Literal {
                    identifier: "1".into(),
                },
            }),
            else_branch: Some(Box::new(Stmt::Print {
                expr: Expr::Literal {
                    identifier: "2".into(),
                },
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
            condition: Expr::Literal {
                identifier: "true".into(),
            },
            body: Box::new(Stmt::Print {
                expr: Expr::Literal {
                    identifier: "1".into(),
                },
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
                    expr: Expr::Literal {
                        identifier: "1".into(),
                    },
                },
                Stmt::Print {
                    expr: Expr::Literal {
                        identifier: "2".into(),
                    },
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
                    value: Some(Expr::Literal {
                        identifier: "a".into(),
                    }),
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
            value: Some(Expr::Literal {
                identifier: "42".into(),
            }),
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
        let expected = Expr::Literal {
            identifier: "\"hello\"".into(),
        };

        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.assignment().unwrap();
        assert_eq!(expr, expected);
    }
}
