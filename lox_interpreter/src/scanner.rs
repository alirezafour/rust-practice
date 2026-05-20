use crate::parser::{Token, TokenTypes};

pub struct Scanner {
    pub source_code: String,
    pub current: usize,
    pub line: usize,
}

impl Scanner {
    pub fn scan_tokens(&mut self) -> Vec<Token> {
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
                        lexeme: format!("\"{}\"", string_literal),
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
