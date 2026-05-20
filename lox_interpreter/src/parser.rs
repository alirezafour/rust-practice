#[derive(Debug, Clone, PartialEq)]
pub enum TokenTypes {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenTypes,
    pub lexeme: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        identifier: String,
    },
    Binary {
        left: Box<Expr>,
        operation: Token,
        right: Box<Expr>,
    },
    Unary {
        operation: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Assign {
        identifier: String,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        logical: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block {
        data: Vec<Stmt>,
    },
    Var {
        name: String,
        value: Option<Expr>,
    },
    Expression {
        expr: Expr,
    },
    Print {
        expr: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    If {
        condition: Expr,
        body: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Return {
        value: Option<Expr>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
    },
    Class {
        name: String,
        functions: Box<Stmt>,
        members: Vec<String>,
    },
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
            && token.token_type == token_type
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
    fn expect(&mut self, token_type: TokenTypes) -> Token {
        let message = format!("expected: {:?}.", &token_type);
        if self.check(token_type) {
            return self.advance();
        } else {
            panic!("{}", message);
        }
    }

    pub fn assignment(&mut self) -> Expr {
        let left = self.or();
        if self.check(TokenTypes::Equal) {
            match left {
                Expr::Literal { identifier } => {
                    let _ = self.advance();
                    let right = self.assignment();
                    return Expr::Assign {
                        identifier,
                        right: Box::new(right),
                    };
                }
                _ => panic!("invalid "),
            }
        }
        left
    }
    fn or(&mut self) -> Expr {
        let left = self.and();
        if self.check(TokenTypes::Or) {
            let logical = self.advance();
            let right = self.or();
            return Expr::Logical {
                left: Box::new(left),
                logical: logical,
                right: Box::new(right),
            };
        }
        left
    }
    fn and(&mut self) -> Expr {
        let left = self.equality();
        if self.check(TokenTypes::And) {
            let logical = self.advance();
            let right = self.and();
            return Expr::Logical {
                left: Box::new(left),
                logical: logical,
                right: Box::new(right),
            };
        }
        left
    }
    fn equality(&mut self) -> Expr {
        let mut left = self.comparison();
        while self.check(TokenTypes::EqualEqual) || self.check(TokenTypes::BangEqual) {
            let operation = self.advance();
            let right = self.comparison();
            left = Expr::Binary {
                left: Box::new(left),
                operation,
                right: Box::new(right),
            }
        }
        left
    }
    fn comparison(&mut self) -> Expr {
        let mut left = self.term();
        while self.check(TokenTypes::Less)
            || self.check(TokenTypes::LessEqual)
            || self.check(TokenTypes::Greater)
            || self.check(TokenTypes::GreaterEqual)
        {
            let operation = self.advance();
            let right = self.term();
            left = Expr::Binary {
                left: Box::new(left),
                operation,
                right: Box::new(right),
            }
        }
        left
    }
    fn term(&mut self) -> Expr {
        let mut left = self.factor();
        while self.check(TokenTypes::Plus) || self.check(TokenTypes::Minus) {
            let operation = self.advance();
            let right = self.factor();
            left = Expr::Binary {
                left: Box::new(left),
                operation,
                right: Box::new(right),
            }
        }
        left
    }
    fn factor(&mut self) -> Expr {
        let mut left = self.unary();
        while self.check(TokenTypes::Star) || self.check(TokenTypes::Slash) {
            let operation = self.advance();
            let right = self.unary();
            left = Expr::Binary {
                left: Box::new(left),
                operation,
                right: Box::new(right),
            }
        }
        left
    }
    fn unary(&mut self) -> Expr {
        let mut left = self.call();
        if self.check(TokenTypes::Minus) || self.check(TokenTypes::Bang) {
            let operation = self.advance();
            let right = self.primary();
            left = Expr::Unary {
                operation: operation,
                right: Box::new(right),
            };
        }
        left
    }
    fn call(&mut self) -> Expr {
        let mut left = self.primary();
        while self.check(TokenTypes::LeftParen) && !self.check(TokenTypes::Eof) {
            if self.check_and_advance(TokenTypes::LeftParen) {
                let paren = self.tokens[self.current - 1].clone();
                let mut arguments = vec![];
                while !self.check(TokenTypes::RightParen) && !self.check(TokenTypes::Eof) {
                    let argument = self.assignment();
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
                    panic!("expected closing paren for function call.");
                }
            }
        }
        left
    }
    fn primary(&mut self) -> Expr {
        if self.check(TokenTypes::Number) || self.check(TokenTypes::Identifier) {
            let literal = self.advance();
            return Expr::Literal {
                identifier: literal.lexeme,
            };
        } else if self.check(TokenTypes::LeftParen) {
            let open_group = self.advance();
            let right = self.equality();
            if self.check(TokenTypes::RightParen) {
                let _ = self.advance();
                return Expr::Grouping {
                    expression: Box::new(right),
                };
            } else {
                panic!("expected closing for {}", open_group.lexeme);
            }
        } else if self.check(TokenTypes::True)
            || self.check(TokenTypes::False)
            || self.check(TokenTypes::Nil)
        {
            let key_word = self.advance();
            return Expr::Literal {
                identifier: key_word.lexeme,
            };
        } else if self.check(TokenTypes::String) {
            let string = self.advance();
            return Expr::Literal {
                identifier: string.lexeme,
            };
        } else if self.check(TokenTypes::Fun) {
            return self.parse_function_expr();
        }
        panic!("failed.");
    }
    fn parse_function_expr(&mut self) -> Expr {
        let left = self.advance();
        if self.check_and_advance(TokenTypes::LeftParen) {
            let mut arguments = vec![];
            while !self.check(TokenTypes::RightParen) && !self.check(TokenTypes::Eof) {
                arguments.push(self.advance().lexeme);
                self.check_and_advance(TokenTypes::Comma);
            }
            self.expect(TokenTypes::RightParen);
            return Expr::Lambda {
                params: arguments,
                body: Box::new(self.parse_block()),
            };
        }
        Expr::Literal {
            identifier: left.lexeme,
        }
    }

    fn check_semicolon(&mut self, statement: Stmt) -> Stmt {
        if self.check(TokenTypes::Semicolon) {
            let _ = self.advance();
            return statement;
        }
        panic!("expected semicolon.")
    }
    pub fn parse_porgram(&mut self) -> Vec<Stmt> {
        let mut v = Vec::new();
        while !self.check(TokenTypes::Eof) {
            v.push(self.parse_statement());
        }
        v
    }
    pub fn parse_statement(&mut self) -> Stmt {
        if self.check_and_advance(TokenTypes::Print) {
            return self.parse_print();
        } else if self.check_and_advance(TokenTypes::Var) {
            return self.parse_var();
        } else if self.check_and_advance(TokenTypes::If) {
            return self.parse_if();
        } else if self.check_and_advance(TokenTypes::While) {
            return self.parse_while();
        } else if self.check(TokenTypes::LeftBrace) {
            return self.parse_block();
        } else if self.check_and_advance(TokenTypes::Return) {
            return self.parse_return();
        } else if self.check_and_advance(TokenTypes::Fun) {
            return self.parse_function();
        }
        return self.parse_expr();
    }
    fn parse_expr(&mut self) -> Stmt {
        let expr = self.assignment();
        return self.check_semicolon(Stmt::Expression { expr });
    }
    fn parse_var(&mut self) -> Stmt {
        let name = self.expect(TokenTypes::Identifier).lexeme;
        let mut value = None;
        if self.check_and_advance(TokenTypes::Equal) {
            value = Some(self.assignment());
        }
        return self.check_semicolon(Stmt::Var { name, value });
    }
    fn parse_print(&mut self) -> Stmt {
        let print_stmt = self.assignment();
        return self.check_semicolon(Stmt::Print { expr: print_stmt });
    }
    fn parse_if(&mut self) -> Stmt {
        self.expect(TokenTypes::LeftParen);
        let expr = self.assignment();
        self.expect(TokenTypes::RightParen);
        let body = self.parse_statement();
        let mut else_branch = None;
        if self.check_and_advance(TokenTypes::Else) {
            else_branch = Some(Box::new(self.parse_statement()));
        }
        return Stmt::If {
            condition: expr,
            body: Box::new(body),
            else_branch: else_branch,
        };
    }
    fn parse_while(&mut self) -> Stmt {
        self.expect(TokenTypes::LeftParen);
        let expr = self.assignment();
        self.expect(TokenTypes::RightParen);
        let body = self.parse_statement();
        return Stmt::While {
            condition: expr,
            body: Box::new(body),
        };
    }
    fn parse_block(&mut self) -> Stmt {
        if !self.check_and_advance(TokenTypes::LeftBrace) {
            panic!("wrong call to parse block.");
        }
        let mut data = vec![];
        while !self.check(TokenTypes::Eof) && !self.check(TokenTypes::RightBrace) {
            data.push(self.parse_statement());
        }
        self.expect(TokenTypes::RightBrace);
        return Stmt::Block { data };
    }
    fn parse_return(&mut self) -> Stmt {
        if self.check_and_advance(TokenTypes::Semicolon) {
            return Stmt::Return { value: None };
        }
        let return_value = self.assignment();
        return self.check_semicolon(Stmt::Return {
            value: Some(return_value),
        });
    }
    fn parse_function(&mut self) -> Stmt {
        let name = self.expect(TokenTypes::Identifier).lexeme;
        self.expect(TokenTypes::LeftParen);
        let mut params = vec![];
        while !self.check(TokenTypes::RightParen) && !self.check(TokenTypes::Eof) {
            if self.check(TokenTypes::Identifier) {
                params.push(self.advance().lexeme);
                self.check_and_advance(TokenTypes::Comma);
            } else {
                panic!(
                    "expected parameter name. but see {:?}",
                    self.advance().lexeme
                );
            }
        }
        self.expect(TokenTypes::RightParen);
        let body = self.parse_block();
        return Stmt::Function {
            name: name,
            params: params,
            body: Box::new(body),
        };
    }
}
