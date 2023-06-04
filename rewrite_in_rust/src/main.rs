mod lexer;

use std::time::Instant;

const SOURCE: &str = "
let a = 1
while a < 10000000 {
    a = a + 1
}
";

fn main() {
    let start = Instant::now();
    lexer::scan(String::from(SOURCE));
    let duration = start.elapsed();
    println!("{:?}", duration.as_secs_f64());
}
