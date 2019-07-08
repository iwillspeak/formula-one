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
        print(eval::eval(read()));
    }
}

/// Read the input string from source and parse it
fn read() -> ast::Expr {
    let mut buff = String::new();
    print!("\u{1F3CE}  > ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut buff).unwrap();
    parse::parse(&buff)
}

/// Print out the result of an expression evaluation
fn print(value: eval::Value) {
    println!(" ~> {}", value);
}
