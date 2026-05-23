use crate::{interpreter::Interpreter, parser::Parser, scanner::Scanner};

mod interpreter;
mod parser;
mod scanner;

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
                column: 0,
            };

            let tokens = scanner.scan_tokens();
            match tokens {
                Ok(tokens) => println!("Tokens: {:?}", tokens),
                Err(err) => println!(
                    "error: {} at line[{}:{}].",
                    &err.message, &err.line, &err.column
                ),
            }
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
            "foo(1)(2)",
            "foo(1,2)",
            "fun(x){return 0;}",
        ];
        for source in test_cases {
            let mut scanner = Scanner {
                source_code: source.to_string(),
                current: 0,
                line: 1,
                column: 0,
            };
            let tokens = scanner.scan_tokens();
            match tokens {
                Ok(tokens) => {
                    let mut parser = Parser::new(tokens);
                    let expr = parser.assignment();
                    match expr {
                        Ok(expr) => println!("==\nexpr: {expr:?}"),
                        Err(err) => println!(
                            "error: {} at line[{}:{}].",
                            err.message, err.token.line, err.token.column
                        ),
                    }
                }
                Err(err) => println!(
                    "error: {} at line[{}:{}].",
                    err.message, err.line, err.column
                ),
            }
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
            "return;",
            "return a*2+2;",
            "fun foo(a,b){ return true;}",
            "var x = fun(x){var x = 0;return x;};",
        ];
        for source in test_cases {
            let mut scanner = Scanner {
                source_code: source.to_string(),
                current: 0,
                line: 1,
                column: 0,
            };
            let tokens = scanner.scan_tokens();
            match tokens {
                Ok(tokens) => {
                    let mut parser = Parser::new(tokens);
                    let stmt = parser.parse_statement();
                    match stmt {
                        Ok(stmt) => println!("==\nstmt: {stmt:?}"),
                        Err(err) => println!(
                            "error: {} at line[{}:{}].",
                            err.message, err.token.line, err.token.column
                        ),
                    }
                }
                Err(err) => println!(
                    "error: {} at line[{}:{}].",
                    err.message, err.line, err.column
                ),
            }
        }
    }
    {
        println!("===\nInterpreter:\n===");
        let test_cases = vec![
            "print 2 + 3;\nprint (4+5)*2;\nprint true;\nvar x = 2 + 3;\nvar check = true;\nif (check) print check;",
            "if (false) print \"this shouldn't print\"; else print \"this should print\";",
            "{var abc = 1;print abc;}",
            "var a = true; a = false; print a;",
            "print \"hello\" and true;\nprint true or false;\nprint false or \"hello world.\";",
            "fun add(a, b) { return a + b; }\nprint add(1, 2);",
            "var f = fun (a,b){ return a+b;};\nprint f(1,2);",
            "var a = true; while(a){a=false;print \"one time print\";}",
            "for(var i=0;i<5;i=i+1) print i;",
        ];
        let mut count = 0;
        for source in test_cases {
            count += 1;
            println!("\ntest {}: \n===", &count);
            let mut scanner = Scanner {
                source_code: source.to_string(),
                current: 0,
                line: 1,
                column: 0,
            };
            let tokens = scanner.scan_tokens();
            match tokens {
                Ok(tokens) => {
                    let mut parser = Parser::new(tokens);
                    let statements = parser.parse_program();
                    match statements {
                        Ok(statements) => {
                            let mut inter = Interpreter::new();
                            for stmt in statements {
                                let result = inter.execute(&stmt);
                                if result.is_err() {
                                    let err = result.err().unwrap();
                                    println!(
                                        "error: {} at line [{}:{}].",
                                        err.message, err.token.line, err.token.column
                                    );
                                }
                            }
                        }
                        Err(err) => println!(
                            "error: {} at line [{}:{}].",
                            err.message, err.token.line, err.token.column
                        ),
                    }
                }
                Err(err) => println!(
                    "error: {} at line[{}:{}].",
                    err.message, err.line, err.column
                ),
            }
        }
    }
}
