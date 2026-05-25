use crate::{interpreter::Interpreter, parser::Parser, scanner::Scanner};

mod interpreter;
mod parser;
mod scanner;

fn main() {
    {
        println!("===\nInterpreter:\n===");
        let test_cases = vec![
            "print 2 + 3;
            print (4+5)*2;
            print true;
            var x = 2 + 3;
            var check = true;
            if (check) print check;",
            //
            "if (false) print \"this shouldn't print\"; else print \"this should print\";",
            //
            "{var abc = 1;print abc;}",
            //
            "var a = true; a = false; print a;",
            //
            "print \"hello\" and true;
            print true or false;
            print false or \"hello world.\";",
            "fun add(a, b) { return a + b; }
            print add(1, 2);",
            //
            "var f = fun (a,b){ return a+b;};
            print f(1,2);",
            //
            "var a = true; while(a){a=false;print \"one time print\";}",
            //
            "for(var i=0;i<5;i=i+1) print i;",
            //
            "fun makeCounter() {
                var count = 0;
                fun counter() {
                  count = count + 1;
                  return count;
                }
                return counter;
              }
            
              var c = makeCounter();
              print c();  // should print 1
              print c();  // should print 2",
            //
        ];
        let mut count = 0;
        for source in test_cases {
            count += 1;
            println!("\ntest {}: \n===", &count);
            let mut scanner = Scanner {
                source_code: source.to_string(),
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
