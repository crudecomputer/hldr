mod analyzer;
mod lexer;
mod loader;
mod parser;

#[allow(unused)]
pub fn place(input: &str) {
    let tokens = lexer::tokenize(input.chars()).unwrap();
    let parse_tree = parser::parse(tokens.into_iter()).unwrap();
    let parse_tree = analyzer::analyze(parse_tree).unwrap();
    loader::load(parse_tree);
}
