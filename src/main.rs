#[deny(missing_docs)]
mod ast;
mod parse;

/// Main Entry Point
///
/// Runs the REPL for the language
fn main() {
    loop {
        println!("{:#?}", read());
    }
}

/// Read the input string from source and parse it
fn read() -> ast::Expr {
    let mut buff = String::new();
    std::io::stdin().read_line(&mut buff).unwrap();
    parse::parse(&buff)
}
