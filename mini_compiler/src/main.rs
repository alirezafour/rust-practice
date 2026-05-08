#[derive(Debug)]
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
}

#[derive(Debug)]
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
                '/' => {
                    let token = Token {
                        token_type: TokenTypes::SLASH,
                        lexeme: "/".to_string(),
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
                _ => {
                    panic!("Unknown character: {}", c);
                }
            }
        }
        tokens
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
