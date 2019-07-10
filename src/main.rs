#[deny(missing_docs)]
mod ast;
mod eval;
mod parse;

use std::fs;
use std::io::prelude::*;

/// Main Entry Point
///
/// Runs the REPL for the language
fn main() {
    let args = std::env::args();
    if args.len() > 1 {
        for arg in args.skip(1) {
            let source = fs::read_to_string(&arg).expect("Could not read source file");
            print(eval::eval(parse::parse(&source)));
        }
    } else {
        let mut env = eval::make_global_env();
        loop {
            print(eval::eval_with_env(read(), &mut env));
        }
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
fn print(result: eval::EvalResult) {
    match result {
        Ok(value) => println!(" ~> {}", value),
        Err(error) => println!(" !! {}", error),
    }
}
