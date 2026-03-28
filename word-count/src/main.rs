use std::{
    collections::HashMap,
    io::{self, Write},
};

fn word_count(input: &str) -> HashMap<&str, u32> {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for word in input.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

fn format_counts(counts: HashMap<&str, u32>) -> String {
    let mut items: Vec<_> = counts.iter().collect();
    items.sort_unstable_by_key(|(_, count)| *count);
    items.reverse();

    let mut result = String::new();
    for (word, count) in items {
        result.push_str(format!("{:>12}\t{}\n", word, count).as_str());
    }
    result
}

fn main() {
    print!("input the sentence: ");
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read line");

    println!("{}", format_counts(word_count(&buf)));
}
