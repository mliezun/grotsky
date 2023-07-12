mod lexer;
mod vm;

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
    vm::test_vm_execution();
    let duration = start.elapsed();
    println!("Duration bytecode: {:?}", duration.as_secs_f64());
    lexer::scan(String::from(source));
}
