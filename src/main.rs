#[deny(missing_docs)]
mod ast;
mod eval;
mod parse;

use std::io::prelude::*;

/// Main Entry Point
///
/// Runs the REPL for the language
fn main() {
    loop {
        let expr = read();
        println!("{:#?}", expr);
        println!("] {:#?}", eval::eval(expr));
    }
}

/// Read the input string from source and parse it
fn read() -> ast::Expr {
    let mut buff = String::new();
    print!("> ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut buff).unwrap();
    parse::parse(&buff)
}
