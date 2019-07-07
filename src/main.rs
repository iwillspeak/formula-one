#[deny(missing_docs)]

mod ast;

/// Main Entry Point
///
/// Runs the REPL for the language
fn main() {
    println!("Test: {:?}", ast::Token::new(ast::TokenKind::LeftBracket));
}
