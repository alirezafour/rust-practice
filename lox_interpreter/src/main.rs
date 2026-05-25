use std::io::Write;

use crate::{
    interpreter::Interpreter,
    parser::Parser,
    scanner::{Scanner, Stmt, Token},
};

mod interpreter;
mod parser;
mod scanner;

fn run(source: &str) -> Vec<Stmt> {
    let mut scanner = Scanner {
        source_code: String::from(source),
        line: 1,
        column: 0,
    };
    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens,
        Err(err) => {
            println!(
                "\nerror: {} in line [{}:{}]",
                err.message, err.line, err.column
            );
            Vec::new()
        }
    };
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(data) => data,
        Err(err) => {
            println!(
                "\nerror: {} in line [{}:{}]",
                err.message, err.token.line, err.token.column
            );
            Vec::new()
        }
    }
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
        let statements = run(&buf);
        for stmt in statements {
            match interpreter.execute(&stmt) {
                Ok(_) => {}
                Err(err) => println!(
                    "\nerror: {} in line [{}:{}]",
                    err.message, err.token.line, err.token.column
                ),
            }
        }
    }
}

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    looped();
}
