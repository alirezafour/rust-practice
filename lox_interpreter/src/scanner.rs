use crate::parser::{Token, TokenTypes};

pub struct ScannerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub struct Scanner {
    pub source_code: String,
    pub line: usize,
    pub column: usize,
}

impl Scanner {
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ScannerError> {
        let mut tokens = Vec::new();
        let mut chars = self.source_code.chars().peekable();
        while let Some(c) = chars.next() {
            self.column += 1;
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                }
                continue;
            }
            let start_column = self.column;
            match c {
                '{' => {
                    let token = Token {
                        token_type: TokenTypes::LeftBrace,
                        lexeme: "{".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                '-' => {
                    let token = Token {
                        token_type: TokenTypes::Minus,
                        lexeme: "-".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                '+' => {
                    let token = Token {
                        token_type: TokenTypes::Plus,
                        lexeme: "+".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                ';' => {
                    let token = Token {
                        token_type: TokenTypes::Semicolon,
                        lexeme: ";".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                '*' => {
                    let token = Token {
                        token_type: TokenTypes::Star,
                        lexeme: "*".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                ',' => {
                    let token = Token {
                        token_type: TokenTypes::Comma,
                        lexeme: ",".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                '}' => {
                    let token = Token {
                        token_type: TokenTypes::RightBrace,
                        lexeme: "}".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                '(' => {
                    let token = Token {
                        token_type: TokenTypes::LeftParen,
                        lexeme: "(".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                ')' => {
                    let token = Token {
                        token_type: TokenTypes::RightParen,
                        lexeme: ")".to_string(),
                        line: self.line,
                        column: start_column,
                    };
                    tokens.push(token);
                }
                '!' => {
                    if let Some(&next) = chars.peek() {
                        if next == '=' {
                            // !=
                            chars.next();
                            self.column += 1;
                            tokens.push(Token {
                                token_type: TokenTypes::BangEqual,
                                lexeme: "!=".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Bang,
                                lexeme: "!".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Bang,
                            lexeme: "!".to_string(),
                            line: self.line,
                            column: start_column,
                        });
                    }
                }
                '<' => {
                    if let Some(&next) = chars.peek() {
                        if next == '\n' {
                            return Err(ScannerError {
                                message: "Unexpected new line.".to_string(),
                                column: self.column,
                                line: self.line,
                            });
                        }
                        if next == '=' {
                            // <=
                            chars.next();
                            self.column += 1;
                            tokens.push(Token {
                                token_type: TokenTypes::LessEqual,
                                lexeme: "<=".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Less,
                                lexeme: "<".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Less,
                            lexeme: "<".to_string(),
                            line: self.line,
                            column: start_column,
                        });
                    }
                }
                '>' => {
                    if let Some(&next) = chars.peek() {
                        if next == '\n' {
                            return Err(ScannerError {
                                message: "Unexpected new line.".to_string(),
                                column: self.column,
                                line: self.line,
                            });
                        }
                        if next == '=' {
                            // >=
                            chars.next();
                            self.column += 1;
                            tokens.push(Token {
                                token_type: TokenTypes::GreaterEqual,
                                lexeme: ">=".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Greater,
                                lexeme: ">".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Greater,
                            lexeme: ">".to_string(),
                            line: self.line,
                            column: start_column,
                        });
                    }
                }
                '=' => {
                    if let Some(&next) = chars.peek() {
                        if next == '=' {
                            // ==
                            chars.next();
                            self.column += 1;
                            tokens.push(Token {
                                token_type: TokenTypes::EqualEqual,
                                lexeme: "==".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Equal,
                                lexeme: "=".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Equal,
                            lexeme: "=".to_string(),
                            line: self.line,
                            column: start_column,
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
                                    self.column += 1;
                                    number_str.push(next);
                                } else {
                                    break;
                                }
                            }
                            tokens.push(Token {
                                token_type: TokenTypes::Number,
                                lexeme: number_str,
                                line: self.line,
                                column: start_column,
                            });
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Dot,
                                lexeme: ".".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Dot,
                            lexeme: ".".to_string(),
                            line: self.line,
                            column: start_column,
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
                            self.column += 1;
                            number_str.push(next);
                        } else if next.is_ascii_alphabetic() {
                            return Err(ScannerError {
                                message: format!("Unexpected character after number: {}", next)
                                    .to_string(),
                                line: self.line,
                                column: self.column,
                            });
                        } else {
                            break;
                        }
                    }
                    if let Some(&next) = chars.peek() {
                        if next == '.' {
                            is_float = true;
                            chars.next();
                            self.column += 1;
                            number_str.push(next);
                            if let Some(&next) = chars.peek() {
                                if next.is_ascii_digit() {
                                    while let Some(&next) = chars.peek() {
                                        if next.is_ascii_digit() {
                                            chars.next();
                                            self.column += 1;
                                            number_str.push(next);
                                        } else if next.is_ascii_alphabetic() {
                                            return Err(ScannerError {
                                                message: std::format!(
                                                    "Unexpected character after number: {}",
                                                    next
                                                )
                                                .to_string(),
                                                column: self.column,
                                                line: self.line,
                                            });
                                        } else {
                                            break;
                                        }
                                    }
                                } else {
                                    return Err(ScannerError {
                                        message: "Expected digit after decimal point in number"
                                            .to_string(),
                                        column: self.column,
                                        line: self.line,
                                    });
                                }
                            }
                        }
                    }
                    if is_float {
                        tokens.push(Token {
                            token_type: TokenTypes::Number,
                            lexeme: number_str,
                            line: self.line,
                            column: start_column,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Number,
                            lexeme: number_str,
                            line: self.line,
                            column: start_column,
                        });
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut identifier_str = String::new();
                    identifier_str.push(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_ascii_alphanumeric() || next == '_' {
                            chars.next();
                            self.column += 1;
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
                        column: start_column,
                    });
                }
                '\"' => {
                    let mut string_literal = String::new();
                    let mut is_valid = false;
                    while let Some(next) = chars.next() {
                        self.column += 1;
                        if next == '\"' {
                            is_valid = true;
                            break;
                        } else if next == '\\' {
                            if let Some(escaped) = chars.next() {
                                self.column += 1;
                                match escaped {
                                    'n' => string_literal.push('\n'),
                                    't' => string_literal.push('\t'),
                                    '\\' => string_literal.push('\\'),
                                    '\"' => string_literal.push('\"'),
                                    _ => {
                                        return Err(ScannerError {
                                            line: self.line,
                                            column: self.column,
                                            message: format!(
                                                "Invalid escape sequence: \\{}",
                                                escaped
                                            ),
                                        });
                                    }
                                }
                            } else {
                                return Err(ScannerError {
                                    message: "Unterminated escape sequence in string literal"
                                        .to_string(),
                                    line: self.line,
                                    column: self.column,
                                });
                            }
                        } else if next == '\n' {
                            return Err(ScannerError {
                                message: "Unexpected new line in string literal".to_string(),
                                line: self.line,
                                column: self.column,
                            });
                        } else {
                            string_literal.push(next);
                        }
                    }
                    if !is_valid {
                        return Err(ScannerError {
                            message: "Unterminated string literal".to_string(),
                            line: self.line,
                            column: self.column,
                        });
                    }
                    tokens.push(Token {
                        token_type: TokenTypes::String,
                        lexeme: format!("\"{}\"", string_literal),
                        line: self.line,
                        column: start_column,
                    });
                }
                '/' => {
                    if let Some(&next) = chars.peek() {
                        if next == '/' {
                            while let Some(next) = chars.next() {
                                self.column += 1;
                                if next == '\n' {
                                    self.line += 1;
                                    self.column = 0;
                                    break;
                                }
                            }
                        } else {
                            tokens.push(Token {
                                token_type: TokenTypes::Slash,
                                lexeme: "/".to_string(),
                                line: self.line,
                                column: start_column,
                            });
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenTypes::Slash,
                            lexeme: '/'.to_string(),
                            line: self.line,
                            column: start_column,
                        });
                    }
                }
                _ => {
                    return Err(ScannerError {
                        line: self.line,
                        column: self.column,
                        message: format!("Unknown character: {}", c),
                    });
                }
            }
        }
        tokens.push(Token {
            token_type: TokenTypes::Eof,
            lexeme: "".to_string(),
            line: self.line,
            column: 0,
        });
        Ok(tokens)
    }
}
