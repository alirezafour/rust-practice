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
            "foo(1)(2)",
            "foo(1,2)",
            "fun(x){return 0;}",
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
            };
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            let stmt = parser.parse_statement();
            println!("==\nstmt: {stmt:?}");
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
            // phase 5 will fix it
            // "var a = true; while(a){a=false;print \"one time print\";}",
            //
        ];
        let mut count = 0;
        for source in test_cases {
            count += 1;
            println!("\ntest {}: \n===", &count);
            let mut scanner = Scanner {
                source_code: source.to_string(),
                current: 0,
                line: 1,
            };
            let tokens = scanner.scan_tokens();

            let mut parser = Parser::new(tokens);
            let statements = parser.parse_porgram();
            let mut inter = Interpreter::new();
            for stmt in statements {
                inter.execute(&stmt);
            }
        }
    }
}
