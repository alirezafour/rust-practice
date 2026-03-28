use std::io::{self, Write};

fn fib(n: u64) -> Option<u64> {
    let (mut a, mut b) = (0, 1);
    for _ in 0..n {
        let tmp: u64 = a;
        a = b;
        b = tmp.checked_add(b)?;
    }
    Some(a)
}

fn print_fib(fibs: &Vec<Option<u64>>) -> String {
    let mut result = String::new();
    result.push_str("[");
    for value in fibs {
        match value {
            Some(v) => result.push_str(format!("{}, ", v).as_str()),
            None => result.push_str("."),
        }
    }
    result.push_str("]");
    result
}

fn main() {
    print!("Enter the number to calculate fibonacci: ");
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("failed to read input.");
    let n = buf.trim().parse::<u64>();
    let fibs: Vec<_> = (0..n.unwrap_or(0)).map(fib).collect();
    println!("{}", print_fib(&fibs));
}
