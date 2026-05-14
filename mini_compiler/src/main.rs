use std::panic;

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

#[derive(Debug, Clone)]
struct Token {
    pub token_type: TokenTypes,
    pub lexeme: String,
    pub line: usize,
}

struct Scanner {
    pub source_code: String,
    pub current: usize,
    pub line: usize,
}

impl Scanner {
    fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = self.source_code.chars().peekable();
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                }
                continue;
            }
            match c {
                '{' => {
                    let token = Token {
                        token_type: TokenTypes::LeftBrace,
                        lexeme: "{".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '-' => {
                    let token = Token {
                        token_type: TokenTypes::Minus,
                        lexeme: "-".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '+' => {
                    let token = Token {
                        token_type: TokenTypes::Plus,
                        lexeme: "+".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                ';' => {
                    let token = Token {
                        token_type: TokenTypes::Semicolon,
                        lexeme: ";".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '*' => {
                    let token = Token {
                        token_type: TokenTypes::Star,
                        lexeme: "*".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                ',' => {
                    let token = Token {
                        token_type: TokenTypes::Comma,
                        lexeme: ",".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '}' => {
                    let token = Token {
                        token_type: TokenTypes::RightBrace,
                        lexeme: "}".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '(' => {
                    let token = Token {
                        token_type: TokenTypes::LeftParen,
                        lexeme: "(".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                ')' => {
                    let token = Token {
                        token_type: TokenTypes::RightParen,
                        lexeme: ")".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '!' => {
                    if let Some(&next) = chars.peek() {
                        if next == '=' {
                            // !=
                            chars.next();
                            tokens.push(Token {
                                token_type: TokenTypes::BangEqual,
                                lexeme: "!=".to_string(),
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Bang,
                                lexeme: "!".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Bang,
                            lexeme: "!".to_string(),
                            line: self.line,
                        });
                    }
                }
                '<' => {
                    if let Some(&next) = chars.peek() {
                        if next == '\n' {
                            panic!("Unexpected new line after <");
                        }
                        if next == '=' {
                            // <=
                            chars.next();
                            tokens.push(Token {
                                token_type: TokenTypes::LessEqual,
                                lexeme: "<=".to_string(),
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Less,
                                lexeme: "<".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Less,
                            lexeme: "<".to_string(),
                            line: self.line,
                        });
                    }
                }
                '>' => {
                    if let Some(&next) = chars.peek() {
                        if next == '\n' {
                            panic!("Unexpected new line after >");
                        }
                        if next == '=' {
                            // >=
                            chars.next();
                            tokens.push(Token {
                                token_type: TokenTypes::GreaterEqual,
                                lexeme: ">=".to_string(),
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Greater,
                                lexeme: ">".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Greater,
                            lexeme: ">".to_string(),
                            line: self.line,
                        });
                    }
                }
                '=' => {
                    if let Some(&next) = chars.peek() {
                        if next == '=' {
                            // ==
                            chars.next();
                            tokens.push(Token {
                                token_type: TokenTypes::EqualEqual,
                                lexeme: "==".to_string(),
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Equal,
                                lexeme: "=".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Equal,
                            lexeme: "=".to_string(),
                            line: self.line,
                        });
                    }
                }
                '.' => {
                    if let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() {
                            let mut number_str = String::new();
                            number_str.push(c);
                            while let Some(&next) = chars.peek() {
                                if next.is_ascii_digit() {
                                    chars.next();
                                    number_str.push(next);
                                } else {
                                    break;
                                }
                            }
                            tokens.push(Token {
                                token_type: TokenTypes::Number,
                                lexeme: number_str,
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Dot,
                                lexeme: ".".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Dot,
                            lexeme: ".".to_string(),
                            line: self.line,
                        });
                    }
                }
                '0'..='9' => {
                    let mut number_str = String::new();
                    let mut is_float = false;
                    number_str.push(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() {
                            chars.next();
                            number_str.push(next);
                        } else if next.is_ascii_alphabetic() {
                            panic!("Unexpected character after number: {}", next);
                        } else {
                            break;
                        }
                    }
                    if let Some(&next) = chars.peek() {
                        if next == '.' {
                            is_float = true;
                            chars.next();
                            number_str.push(next);
                            if let Some(&next) = chars.peek() {
                                if next.is_ascii_digit() {
                                    while let Some(&next) = chars.peek() {
                                        if next.is_ascii_digit() {
                                            chars.next();
                                            number_str.push(next);
                                        } else if next.is_ascii_alphabetic() {
                                            panic!("Unexpected character after number: {}", next);
                                        } else {
                                            break;
                                        }
                                    }
                                } else {
                                    panic!("Expected digit after decimal point in number");
                                }
                            }
                        }
                    }
                    if is_float {
                        tokens.push(Token {
                            token_type: TokenTypes::Number,
                            lexeme: number_str,
                            line: self.line,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Number,
                            lexeme: number_str,
                            line: self.line,
                        });
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut identifier_str = String::new();
                    identifier_str.push(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_ascii_alphanumeric() || next == '_' {
                            chars.next();
                            identifier_str.push(next);
                        } else {
                            break;
                        }
                    }
                    let token_type = match &identifier_str[..] {
                        "and" => TokenTypes::And,
                        "or" => TokenTypes::Or,
                        "class" => TokenTypes::Class,
                        "if" => TokenTypes::If,
                        "else" => TokenTypes::Else,
                        "true" => TokenTypes::True,
                        "false" => TokenTypes::False,
                        "fun" => TokenTypes::Fun,
                        "for" => TokenTypes::For,
                        "while" => TokenTypes::While,
                        "nil" => TokenTypes::Nil,
                        "print" => TokenTypes::Print,
                        "return" => TokenTypes::Return,
                        "super" => TokenTypes::Super,
                        "this" => TokenTypes::This,
                        "var" => TokenTypes::Var,
                        _ => TokenTypes::Identifier,
                    };
                    tokens.push(Token {
                        token_type: token_type,
                        lexeme: identifier_str,
                        line: self.line,
                    });
                }
                '\"' => {
                    let mut string_literal = String::new();
                    let mut is_valid = false;
                    while let Some(next) = chars.next() {
                        if next == '\"' {
                            is_valid = true;
                            break;
                        } else if next == '\\' {
                            if let Some(escaped) = chars.next() {
                                match escaped {
                                    'n' => string_literal.push('\n'),
                                    't' => string_literal.push('\t'),
                                    '\\' => string_literal.push('\\'),
                                    '\"' => string_literal.push('\"'),
                                    _ => panic!("Invalid escape sequence: \\{}", escaped),
                                }
                            } else {
                                panic!("Unterminated escape sequence in string literal");
                            }
                        } else if next == '\n' {
                            panic!("Unexpected new line in string literal");
                        } else {
                            string_literal.push(next);
                        }
                    }
                    if !is_valid {
                        panic!("Unterminated string literal");
                    }
                    tokens.push(Token {
                        token_type: TokenTypes::String,
                        lexeme: string_literal,
                        line: self.line,
                    });
                }
                '/' => {
                    if let Some(&next) = chars.peek() {
                        if next == '/' {
                            while let Some(next) = chars.next() {
                                if next == '\n' {
                                    self.line += 1;
                                    break;
                                }
                            }
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Slash,
                                lexeme: "/".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Slash,
                            lexeme: '/'.to_string(),
                            line: self.line,
                        });
                    }
                }
                _ => {
                    panic!("Unknown character: {}", c);
                }
            }
        }
        tokens.push(Token {
            token_type: TokenTypes::Eof,
            lexeme: "".to_string(),
            line: self.line,
        });
        tokens
    }
}

#[derive(Debug)]
enum Expr {
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
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
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

    fn assignment(&mut self) -> Expr {
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
        let mut left = self.primary();
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
        }
        panic!("failed.");
    }

    fn parse_statement(&mut self) -> Stmt {
        if self.check_and_advance(TokenTypes::Print) {
            let print_stmt = self.assignment();
            return self.check_semicolon(Stmt::Print { expr: print_stmt });
        } else if self.check_and_advance(TokenTypes::Var) {
            let name = self.expect(TokenTypes::Identifier).lexeme;
            let mut value = None;
            if self.check_and_advance(TokenTypes::Equal) {
                value = Some(self.assignment());
            }
            return self.check_semicolon(Stmt::Var { name, value });
        } else if self.check_and_advance(TokenTypes::If) {
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
        } else if self.check_and_advance(TokenTypes::While) {
            self.expect(TokenTypes::LeftParen);
            let expr = self.assignment();
            self.expect(TokenTypes::RightParen);
            let body = self.parse_statement();
            return Stmt::While {
                condition: expr,
                body: Box::new(body),
            };
        } else if self.check_and_advance(TokenTypes::LeftBrace) {
            let mut data = vec![];
            while !self.check(TokenTypes::Eof) && !self.check(TokenTypes::RightBrace) {
                data.push(self.parse_statement());
            }
            self.expect(TokenTypes::RightBrace);
            return Stmt::Block { data };
        } else if self.check_and_advance(TokenTypes::Return) {
            let return_value = self.assignment();
            return self.check_semicolon(Stmt::Return {
                value: return_value,
            });
        }

        let expr = self.assignment();
        return self.check_semicolon(Stmt::Expression { expr });
    }
    fn check_semicolon(&mut self, statement: Stmt) -> Stmt {
        if self.check(TokenTypes::Semicolon) {
            let _ = self.advance();
            return statement;
        }
        panic!("expected semicolon.")
    }
}

#[derive(Debug)]
enum Stmt {
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
        value: Expr,
    },
    Function {
        params: Vec<String>,
        body: Box<Stmt>,
    },
    Class {
        name: String,
        functions: Box<Stmt>,
        members: Vec<String>,
    },
}

fn main() {
    {
        let test_cases = vec![
            "!=",
            "!",
            "<=",
            "<",
            ">=",
            ">",
            "==",
            "=",
            "12.2",
            "12",
            ".12",
            "*",
            "-",
            "+ ",
            ".",
            "\n*12+3128-832=29.2 .12/21!=12",
            "\"abcde\"",
            "abc",
            "var abc = if while\n true == false \")#$ Margarita if else @\"",
            "var x = 10;",
            "var x = 10;\nabc=\"\\n \\t if that all string\"\n// this is just a comment and will be skipped.\na = 2;",
            "x.foo",
            "vv.",
            "abc/",
            "12/221.2",
            "12/ntnt/.",
            "12.",
        ];
        for test in test_cases {
            let source = test.to_string();
            let mut scanner = Scanner {
                source_code: source,
                current: 0,
                line: 1,
            };

            let tokens = scanner.scan_tokens();
            println!("Tokens: {:?}", tokens);
        }
    }
    {
        println!("================\n\n");

        let test_cases = vec![
            "2 == 3",
            "2==3",
            "12+3/2==12-2121",
            "(4+5)*2",
            "true == \"abcde\"",
            "true and false",
            "x = 3",
            "x = 2 or true",
        ];
        for source in test_cases {
            let mut scanner = Scanner {
                source_code: source.to_string(),
                current: 0,
                line: 1,
            };
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            let expr = parser.assignment();
            println!("==\nexpr: {expr:?}");
        }
    }

    {
        println!("===\nStatements:\n===");
        let test_cases = vec![
            "print 2 + 3;",
            "print (4+5)*2;",
            "print true == \"abcde\";",
            "print true and false;",
            "2+2;",
            "var x = 2 + 3;",
            "var x;",
            "var check = true and true;",
            "if (true) print 1;",
            "while(true) var x;",
            "{var abc = 1;}",
            "while (true) { 2+2;}",
        ];
        for source in test_cases {
            let mut scanner = Scanner {
                source_code: source.to_string(),
                current: 0,
                line: 1,
            };
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            let stmt = parser.parse_statement();
            println!("==\nstmt: {stmt:?}");
        }
    }
}
