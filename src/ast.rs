//! Formula One Syntax Tree
//!
//! This module contains the types which define the syntax tree for
//! the language. It's basically an `enum` with the possible node
//! types.
//!
//! The LISP we have to parse is fairly simplified. Token wise we only have:
//!
//!  * `(` and `)` - puncutation
//!  * `[0-9]+` - number literals
//!  * Everything else is a symbol
//!
//! Tokens do however contain a list of leading and trailing trivia
//! which can include whitepace and comments.
//!
//! Expression wise we have the following forms:
//!
//!  * `<symbol>` - reference to the variable `<symbol>`
//!  * `<number>` - reference to a numeric literal
//!  * `(if <cond> <then> <else>)` - condition expression.
//!  * `(define <symbol> <expr>)` - defines a variable to a given
//!                                 value
//!  * `(<symbol> <arg>...)` - Procedure call to `<symbol>`

use codespan::*;

/// A single lexical token in the source text
///
/// Each token represents a single logocal item in the source text. A
/// token is made up of four things:
///
///  * `kind` - the type of token
///  * `span` - the location of the token in the text
///  * `leading_triva` - the token trivia immediately before this token
///  * `trailing_trivia` - the trivia after this token to the end of line
#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    span: Span,
}

/// Datum for the four kinds of token
#[derive(Debug, PartialEq)]
pub enum TokenKind {
    /// The token is the `(` bracket
    LeftBracket,
    /// The token is the `)` bracket
    RightBracket,
    /// The token is a numeric literal
    Number(i64),
    /// The token is an unnamed symbol
    Symbol(String),
}

impl Token {
    /// Create a token with the given `kind` and `span`
    pub fn with_span(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

/// Syntax expression enum
///
/// Represnts one of the expression forms in the lanauge.
#[derive(Debug, PartialEq)]
pub enum Expr {
    /// A direct reference to a variable symbol
    Symbol(Token, String),
    /// A numeric literal
    Number(Token, i64),
    /// A conditional expression
    If(Token, Token, Box<Expr>, Box<Expr>, Box<Expr>, Token),
    /// A variable declaration
    Define(Token, Token, Token, Box<Expr>, Token),
    /// A funciton call expression
    Call(Token, Token, Vec<Expr>, Token),
}
