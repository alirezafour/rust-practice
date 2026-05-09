#[derive(Debug, Clone, PartialEq)]
pub enum TokenTypes {
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,
    IDENTIFIER,
    STRING,
    NUMBER,
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,
    EOF,
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
                        token_type: TokenTypes::LEFT_BRACE,
                        lexeme: "{".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '-' => {
                    let token = Token {
                        token_type: TokenTypes::MINUS,
                        lexeme: "-".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '+' => {
                    let token = Token {
                        token_type: TokenTypes::PLUS,
                        lexeme: "+".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                ';' => {
                    let token = Token {
                        token_type: TokenTypes::SEMICOLON,
                        lexeme: ";".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '*' => {
                    let token = Token {
                        token_type: TokenTypes::STAR,
                        lexeme: "*".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                ',' => {
                    let token = Token {
                        token_type: TokenTypes::COMMA,
                        lexeme: ",".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '}' => {
                    let token = Token {
                        token_type: TokenTypes::RIGHT_BRACE,
                        lexeme: "}".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                '(' => {
                    let token = Token {
                        token_type: TokenTypes::LEFT_PAREN,
                        lexeme: "(".to_string(),
                        line: self.line,
                    };
                    tokens.push(token);
                }
                ')' => {
                    let token = Token {
                        token_type: TokenTypes::RIGHT_PAREN,
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
                                token_type: TokenTypes::BANG_EQUAL,
                                lexeme: "!=".to_string(),
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::BANG,
                                lexeme: "!".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::BANG,
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
                                token_type: TokenTypes::LESS_EQUAL,
                                lexeme: "<=".to_string(),
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::LESS,
                                lexeme: "<".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::LESS,
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
                                token_type: TokenTypes::GREATER_EQUAL,
                                lexeme: ">=".to_string(),
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::GREATER,
                                lexeme: ">".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::GREATER,
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
                                token_type: TokenTypes::EQUAL_EQUAL,
                                lexeme: "==".to_string(),
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::EQUAL,
                                lexeme: "=".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::EQUAL,
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
                                token_type: TokenTypes::NUMBER,
                                lexeme: number_str,
                                line: self.line,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::DOT,
                                lexeme: ".".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::DOT,
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
                            token_type: TokenTypes::NUMBER,
                            lexeme: number_str,
                            line: self.line,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::NUMBER,
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
                        "and" => TokenTypes::AND,
                        "or" => TokenTypes::OR,
                        "class" => TokenTypes::CLASS,
                        "if" => TokenTypes::IF,
                        "else" => TokenTypes::ELSE,
                        "true" => TokenTypes::TRUE,
                        "false" => TokenTypes::FALSE,
                        "fun" => TokenTypes::FUN,
                        "for" => TokenTypes::FOR,
                        "while" => TokenTypes::WHILE,
                        "nil" => TokenTypes::NIL,
                        "print" => TokenTypes::PRINT,
                        "return" => TokenTypes::RETURN,
                        "super" => TokenTypes::SUPER,
                        "this" => TokenTypes::THIS,
                        "var" => TokenTypes::VAR,
                        _ => TokenTypes::IDENTIFIER,
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
                        token_type: TokenTypes::STRING,
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
                                token_type: TokenTypes::SLASH,
                                lexeme: "/".to_string(),
                                line: self.line,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::SLASH,
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
            token_type: TokenTypes::EOF,
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
    fn equality(&mut self) -> Expr {
        let mut left = self.comparison();
        while self.check(TokenTypes::EQUAL_EQUAL) || self.check(TokenTypes::BANG_EQUAL) {
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
        while self.check(TokenTypes::LESS)
            || self.check(TokenTypes::LESS_EQUAL)
            || self.check(TokenTypes::GREATER)
            || self.check(TokenTypes::GREATER_EQUAL)
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
        while self.check(TokenTypes::PLUS) || self.check(TokenTypes::MINUS) {
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
        while self.check(TokenTypes::STAR) || self.check(TokenTypes::SLASH) {
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
        if self.check(TokenTypes::MINUS) || self.check(TokenTypes::BANG) {
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
        if self.check(TokenTypes::NUMBER) || self.check(TokenTypes::IDENTIFIER) {
            let operation = self.advance();
            return Expr::Literal {
                identifier: operation.lexeme,
            };
        }
        panic!("this should be primary");
    }
}

fn main() {
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
    println!("================\n\n");

    let test_cases = vec!["2 == 3", "2==3", "12+3/2==12-2121"];
    for source in test_cases {
        let mut scanner = Scanner {
            source_code: source.to_string(),
            current: 0,
            line: 1,
        };
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let expr = parser.equality();
        println!("==\nexpr: {expr:?}");
    }
}
