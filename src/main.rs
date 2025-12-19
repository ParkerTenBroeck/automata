use automata::{lexer::Lexer, parser::Parser};

fn main() {
    let input = include_str!("../example.txt");
    
    println!("{:#?}", Parser::new(Lexer::new(input)).parse_elements());
}
