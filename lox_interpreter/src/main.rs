use std::io::Write;

use crate::{
    interpreter::Interpreter,
    parser::Parser,
    scanner::{Scanner, Stmt},
};

mod interpreter;
mod parser;
mod scanner;

fn get_statements(source: &str) -> Result<Vec<Stmt>, Box<dyn std::error::Error>> {
    let mut scanner = Scanner {
        source_code: String::from(source),
        line: 1,
        column: 0,
    };
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    Ok(parser.parse_program()?)
}

fn looped() {
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        std::io::stdout().flush().expect("failed to flush stdout.");

        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("failed to read line.");
        let statements = match get_statements(&buf) {
            Ok(stmts) => stmts,
            Err(err) => {
                eprintln!("{err}");
                Vec::new()
            }
        };
        for stmt in statements {
            match interpreter.execute(&stmt) {
                Ok(_) => {}
                Err(err) => eprintln!("{err}"),
            }
        }
    }
}

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    looped();
}
