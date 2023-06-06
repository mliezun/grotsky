mod lexer;

use std::env;
use std::fs::read_to_string;
use std::time::Instant;

const SOURCE: &str = "
let a = 1
while a < 10000000 {
    a = a + 1
}
";

fn main() {
    let args: Vec<String> = env::args().collect();
    let content: String;
    let source: &str;
    if args.len() > 1 {
        content = read_to_string(&args[1]).unwrap();
        source = content.as_str();
    } else {
        source = SOURCE;
    }
    let start = Instant::now();
    lexer::scan(String::from(source));
    let duration = start.elapsed();
    println!("{:?}", duration.as_secs_f64());
}
