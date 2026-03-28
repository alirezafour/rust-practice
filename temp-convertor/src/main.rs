use std::io::{self, Write};

fn c_to_f(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn f_to_c(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn convert_input(input: &str) -> Result<String, String> {
    let mut parts = input.trim().split_whitespace();
    let value = parts.next().ok_or("Missing number")?;
    let unit = parts.next().ok_or("Missing unit")?;
    let num: f64 = value.parse().map_err(|_| "Invalid number")?;

    let result = match unit {
        "c" | "C" => format!("{:.2} F", c_to_f(num)),
        "f" | "F" => format!("{:.2} C", f_to_c(num)),
        _ => return Err("Invalid unit (use 'c' or 'f')".into()),
    };

    Ok(result)
}

fn main() {
    // intro
    print!("Enter value (e.g. 12.2 c): ");
    io::stdout().flush().unwrap();

    // read input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input.");
    match convert_input(&input) {
        Ok(result) => println!("Converted: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
