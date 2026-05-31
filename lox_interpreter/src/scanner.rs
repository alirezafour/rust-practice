use std::collections::HashMap;

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
    pub column: usize,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.lexeme)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        identifier: Token,
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
        identifier: Token,
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
    Set {
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: String,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal { identifier } => write!(f, "{}", identifier),
            Expr::Binary {
                left,
                operation,
                right,
            } => write!(f, "({} {} {})", operation, left.as_ref(), right.as_ref()),
            Expr::Unary { operation, right } => write!(f, "({} {})", operation, right.as_ref()),
            Expr::Grouping { expression } => write!(f, "(group {})", expression.as_ref()),
            Expr::Assign { identifier, right } => write!(f, "(= {} {})", identifier, right),
            Expr::Logical {
                left,
                logical,
                right,
            } => write!(f, "({} {} {})", logical, left, right),
            Expr::Call {
                callee, arguments, ..
            } => {
                let params = arguments
                    .iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "(call {} {})", callee.as_ref(), params)
            }
            Expr::Lambda { params, body } => {
                write!(f, "(lambda {} {})", params.join(" "), body.as_ref())
            }
            Expr::Set {
                object,
                name,
                value,
            } => write!(f, "(set {} {} {})", object.as_ref(), &name, value.as_ref()),
            Expr::Get { object, name } => write!(f, "(get {} {})", object.as_ref(), name),
        }
    }
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
        methods: HashMap<String, Box<Stmt>>,
    },
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Print { expr } => write!(f, "(print {})", expr),
            Stmt::Var { name, value } => {
                let val = match value {
                    Some(v) => format!(" {}", v),
                    None => "".into(),
                };
                write!(f, "(var {}{})", name, val)
            }
            Stmt::Expression { expr } => write!(f, "(expr {})", expr),
            Stmt::Block { data } => {
                let data_out = data
                    .iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "(block {})", data_out)
            }
            Stmt::If {
                condition,
                body,
                else_branch,
            } => {
                let else_out = match else_branch {
                    Some(v) => format!(" else {}", v),
                    None => "".into(),
                };
                write!(f, "(if {} {}{})", condition, body.as_ref(), else_out)
            }
            Stmt::While { condition, body } => write!(f, "(while {} {})", condition, body.as_ref()),
            Stmt::Function { name, params, body } => {
                write!(f, "(fun {} {} {})", name, params.join(" "), body.as_ref())
            }
            Stmt::Return { value } => {
                let val = match value {
                    Some(v) => format!(" {}", v),
                    None => "".into(),
                };
                write!(f, "(return{})", val)
            }
            Stmt::Class { name, methods } => {
                let mut function_names = String::new();
                for ett in methods {
                    match ett.1.as_ref() {
                        Stmt::Function { name, .. } => {
                            function_names.push_str(name);
                            function_names.push(' ');
                        }
                        _ => unreachable!("this is never happen."),
                    }
                }
                write!(f, "(class {} {})", name, function_names)
            }
        }
    }
}

#[derive(Debug)]
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
                    tokens.push(Token {
                        token_type: TokenTypes::Number,
                        lexeme: number_str,
                        line: self.line,
                        column: start_column,
                    });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_number() {
        let mut scanner = Scanner {
            source_code: "12.5".into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 2); // Number + EOF
        assert_eq!(tokens[0].token_type, TokenTypes::Number);
        assert_eq!(tokens[0].lexeme, "12.5");

        let mut scanner = Scanner {
            source_code: "12".into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 2); // Number + EOF
        assert_eq!(tokens[0].token_type, TokenTypes::Number);
        assert_eq!(tokens[0].lexeme, "12");
    }

    #[test]
    fn scans_identifier() {
        let mut scanner = Scanner {
            source_code: "abc".into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenTypes::Identifier);
        assert_eq!(tokens[0].lexeme, "abc");
    }

    #[test]
    fn scans_reserved() {
        let source = "if while for var".to_string();
        let expected_tokens = vec![
            TokenTypes::If,
            TokenTypes::While,
            TokenTypes::For,
            TokenTypes::Var,
        ];
        let split: Vec<&str> = source.split(" ").collect::<Vec<_>>();
        let mut scanner = Scanner {
            source_code: source.clone(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), expected_tokens.len() + 1);
        for idx in 0..expected_tokens.len() {
            assert_eq!(tokens[idx].token_type, expected_tokens[idx]);
            assert_eq!(tokens[idx].lexeme, split[idx]);
        }
    }

    #[test]
    fn scans_brace_and_paren() {
        let source = "( ) { }".to_string();
        let expected_tokens = vec![
            TokenTypes::LeftParen,
            TokenTypes::RightParen,
            TokenTypes::LeftBrace,
            TokenTypes::RightBrace,
        ];
        let split: Vec<&str> = source.split(" ").collect::<Vec<_>>();
        let mut scanner = Scanner {
            source_code: source.clone(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), expected_tokens.len() + 1);
        for idx in 0..expected_tokens.len() {
            assert_eq!(tokens[idx].token_type, expected_tokens[idx]);
            assert_eq!(tokens[idx].lexeme, split[idx]);
        }
    }

    #[test]
    fn scans_symbols() {
        let source = ", . - + ; / * ! != = == > >= < <=".to_string();
        let expected_tokens = vec![
            TokenTypes::Comma,
            TokenTypes::Dot,
            TokenTypes::Minus,
            TokenTypes::Plus,
            TokenTypes::Semicolon,
            TokenTypes::Slash,
            TokenTypes::Star,
            TokenTypes::Bang,
            TokenTypes::BangEqual,
            TokenTypes::Equal,
            TokenTypes::EqualEqual,
            TokenTypes::Greater,
            TokenTypes::GreaterEqual,
            TokenTypes::Less,
            TokenTypes::LessEqual,
        ];
        let split: Vec<&str> = source.split(" ").collect::<Vec<_>>();
        let mut scanner = Scanner {
            source_code: source.clone(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), expected_tokens.len() + 1);
        for idx in 0..expected_tokens.len() {
            assert_eq!(tokens[idx].token_type, expected_tokens[idx]);
            assert_eq!(tokens[idx].lexeme, split[idx]);
        }
    }

    #[test]
    fn scans_string() {
        let mut scanner = Scanner {
            source_code: "\"hello world\"".into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 2); // String + EOF
        assert_eq!(tokens[0].token_type, TokenTypes::String);
        assert_eq!(tokens[0].lexeme, "\"hello world\"");
    }

    #[test]
    fn scans_string_with_escapes() {
        let mut scanner = Scanner {
            source_code: "\"line1\\nline2\"".into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenTypes::String);
        assert_eq!(tokens[0].lexeme, "\"line1\nline2\"");
    }

    #[test]
    fn scans_all_keywords() {
        let source =
            "and class else false fun for if nil or print return super this true var while"
                .to_string();
        let expected_tokens = vec![
            TokenTypes::And,
            TokenTypes::Class,
            TokenTypes::Else,
            TokenTypes::False,
            TokenTypes::Fun,
            TokenTypes::For,
            TokenTypes::If,
            TokenTypes::Nil,
            TokenTypes::Or,
            TokenTypes::Print,
            TokenTypes::Return,
            TokenTypes::Super,
            TokenTypes::This,
            TokenTypes::True,
            TokenTypes::Var,
            TokenTypes::While,
        ];
        let split: Vec<&str> = source.split(" ").collect::<Vec<_>>();
        let mut scanner = Scanner {
            source_code: source.clone(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), expected_tokens.len() + 1); // keywords + EOF
        for idx in 0..expected_tokens.len() {
            assert_eq!(tokens[idx].token_type, expected_tokens[idx]);
            assert_eq!(tokens[idx].lexeme, split[idx]);
        }
    }

    #[test]
    fn scans_multiple_numbers() {
        let source = "1 2.5 100 0.0".to_string();
        let expected_lexemes = vec!["1", "2.5", "100", "0.0"];
        let mut scanner = Scanner {
            source_code: source,
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 5); // 4 numbers + EOF
        for idx in 0..4 {
            assert_eq!(tokens[idx].token_type, TokenTypes::Number);
            assert_eq!(tokens[idx].lexeme, expected_lexemes[idx]);
        }
    }

    #[test]
    fn scans_line_tracking() {
        let source = "1\n2\n3".to_string();
        let mut scanner = Scanner {
            source_code: source,
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 4); // 3 numbers + EOF
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].lexeme, "1");
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].lexeme, "2");
        assert_eq!(tokens[2].line, 3);
        assert_eq!(tokens[2].lexeme, "3");
    }

    #[test]
    fn scans_comments() {
        let source = "42 // this is a comment".to_string();
        let mut scanner = Scanner {
            source_code: source,
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 2); // Number + EOF (comment is skipped)
        assert_eq!(tokens[0].token_type, TokenTypes::Number);
        assert_eq!(tokens[0].lexeme, "42");
    }

    #[test]
    fn scans_whitespace() {
        let source = "  42  ".to_string();
        let mut scanner = Scanner {
            source_code: source,
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 2); // Number + EOF
        assert_eq!(tokens[0].token_type, TokenTypes::Number);
        assert_eq!(tokens[0].lexeme, "42");
    }

    #[test]
    fn scans_eof() {
        let mut scanner = Scanner {
            source_code: "42".into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert!(tokens.len() >= 1);
        assert_eq!(tokens.last().unwrap().token_type, TokenTypes::Eof);
    }

    #[test]
    fn scans_empty_source() {
        let mut scanner = Scanner {
            source_code: "".into(),
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), 1); // only EOF
        assert_eq!(tokens[0].token_type, TokenTypes::Eof);
    }

    #[test]
    fn scans_expression() {
        let source = "1 + 2 * 3".to_string();
        let expected_types = vec![
            TokenTypes::Number,
            TokenTypes::Plus,
            TokenTypes::Number,
            TokenTypes::Star,
            TokenTypes::Number,
            TokenTypes::Eof,
        ];
        let expected_lexemes = vec!["1", "+", "2", "*", "3", ""];
        let mut scanner = Scanner {
            source_code: source,
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), expected_types.len());
        for idx in 0..expected_types.len() {
            assert_eq!(tokens[idx].token_type, expected_types[idx]);
            assert_eq!(tokens[idx].lexeme, expected_lexemes[idx]);
        }
    }

    #[test]
    fn scans_set_get() {
        let source = "x.y = 12".to_string();
        let expected_types = vec![
            TokenTypes::Identifier,
            TokenTypes::Dot,
            TokenTypes::Identifier,
            TokenTypes::Equal,
            TokenTypes::Number,
            TokenTypes::Eof,
        ];
        let expected_lexemes = vec!["x", ".", "y", "=", "12", ""];
        let mut scanner = Scanner {
            source_code: source,
            line: 1,
            column: 0,
        };
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(tokens.len(), expected_types.len());
        for idx in 0..expected_types.len() {
            assert_eq!(tokens[idx].token_type, expected_types[idx]);
            assert_eq!(tokens[idx].lexeme, expected_lexemes[idx]);
        }
    }

    // --- Negative (error) tests ---

    fn assert_scan_error(source: &str, expected_substring: &str) {
        let mut scanner = Scanner {
            source_code: source.into(),
            line: 1,
            column: 0,
        };
        let result = scanner.scan_tokens();
        assert!(
            result.is_err(),
            "expected error but got tokens: {:?}",
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
    fn scan_error_unterminated_string() {
        assert_scan_error("\"hello", "Unterminated string literal");
    }

    #[test]
    fn scan_error_unterminated_string_multiline() {
        assert_scan_error("\"line1\n", "Unexpected new line in string literal");
    }

    #[test]
    fn scan_error_invalid_escape() {
        assert_scan_error("\"\\q\"", "Invalid escape sequence");
    }

    #[test]
    fn scan_error_unknown_char() {
        assert_scan_error("@", "Unknown character");
    }

    #[test]
    fn scan_error_number_then_letter() {
        assert_scan_error("12.5abc", "Unexpected character after number");
    }

    #[test]
    fn scan_error_unexpected_newline_in_escape() {
        assert_scan_error("\"test\\", "Unterminated escape sequence");
    }
}
