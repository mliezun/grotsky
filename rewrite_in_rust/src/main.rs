mod lexer;

use std::time::{Duration, Instant};

const SOURCE: &str = "
let a = 1
while a < 100000000 {
    a = a + 1
}
";

fn main() {
    let start = Instant::now();
    lexer::scan(String::from(SOURCE));
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
}
