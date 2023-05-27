mod lexer;

const SOURCE: &str = "
let list = [[1], 2, 3] # guaska
io.println(list[0][0])
";

fn main() {
    lexer::scan(String::from(SOURCE));
}
