//! Syntax Parser
//!
//! The syntax parser is responsible for taking buffers of characters
//! and returning structured syntax trees.

use super::ast;
use codespan::*;

/// Tokenisation state
///
/// Each variant represents a state in the DFA used by the tokeniser
/// to recognise source text.
enum TokeniseState {
    /// Initial token state. This is not a final state.
    Start,
    /// left parenthesis seen. This is a final state
    Lparen,
    /// Right parenthesis seen. This is a final state
    Rparen,
    /// One or more digits seen. This is a final state.
    Number,
    /// One or more symbol characters seen. This is a final state.
    Symbol,
    /// Unicode whitespace characters
    Whitespace,
}

/// Parser state structure
///
/// Contains the lookahead inforation for the parser
struct ParseState<I: Iterator<Item = ast::Token>>(std::iter::Peekable<I>);

impl<I> ParseState<I>
where
    I: Iterator<Item = ast::Token>,
{
    /// Pase a single form from a list of tokens
    fn parse_expr(&mut self) -> ast::Expr {
        if let Some(token) = self.0.next() {
            use ast::TokenKind::*;
            match token.kind {
                LeftBracket => self.parse_form(token),
                RightBracket => panic!("unexpected token!"),
                Number(_) => ast::Expr::Number(token),
                Symbol(_) => ast::Expr::Symbol(token),
            }
        } else {
            panic!("invalid expression.")
        }
    }

    // Parse one of our recognised strucutred forms beginning with the
    // given token
    fn parse_form(&mut self, open: ast::Token) -> ast::Expr {
        use ast::TokenKind::*;
        match self.0.peek() {
            Some(&ast::Token {
                kind: Symbol(ref sym),
                ..
            }) => match &sym[..] {
                "if" => {
                    let if_tok = self.0.next().unwrap();
                    let cond = self.parse_expr();
                    let if_true = self.parse_expr();
                    let if_false = self.parse_expr();
                    let close = self.0.next().unwrap();
                    ast::Expr::If(
                        open,
                        if_tok,
                        Box::new(cond),
                        Box::new(if_true),
                        Box::new(if_false),
                        close,
                    )
                }
                "define" => {
                    let sym_tok = self.0.next().unwrap();
                    let value = self.parse_expr();
                    let close = self.0.next().unwrap();
                    ast::Expr::Deine(open, sym_tok, Box::new(value), close)
                }
                _ => {
                    let sym_tok = self.0.next().unwrap();
                    let mut args = Vec::new();
                    while let Some(token) = self.0.peek() {
                        if token.kind == RightBracket {
                            break;
                        }
                        args.push(self.parse_expr());
                    }
                    let close = self.0.next().unwrap();
                    ast::Expr::Call(open, sym_tok, args, close)
                }
            },
            _ => panic!("invalid expression"),
        }
    }
}

/// Parse source text into a structured AST expression
///
/// This first tokenises the source text and then parses the resulting
/// list of tokens into a single expression form.
pub fn parse(source: &str) -> ast::Expr {
    let tokens = tokenise(source);
    println!("tokens: {:#?}", tokens);
    ParseState(tokens.into_iter().peekable()).parse_expr()
}

/// Tokenise a given string
fn tokenise(source: &str) -> Vec<ast::Token> {
    use TokeniseState::*;

    let mut result = Vec::new();

    let mut start = 0;

    loop {
        let mut state = Start;
        let mut end = start;

        for c in source[start..].chars() {
            let next = match state {
                Start => match c {
                    '(' => Some(Lparen),
                    ')' => Some(Rparen),
                    '0'...'9' => Some(Number),
                    'a'...'z' => Some(Symbol),
                    c if c.is_whitespace() => Some(Whitespace),
                    _ => None,
                },
                Lparen | Rparen => None,
                Number => match c {
                    '0'...'9' => Some(Number),
                    _ => None,
                },
                Symbol => match c {
                    'a'...'z' => Some(Symbol),
                    _ => None,
                },
                Whitespace => {
                    if c.is_whitespace() {
                        Some(Whitespace)
                    } else {
                        None
                    }
                }
            };

            if let Some(next_state) = next {
                state = next_state;
                end += c.len_utf8();
            } else {
                break;
            }
        }

        let token_str = &source[start..end];
        let span = Span::new(start, end);

        start = end;

        let kind = match state {
            Start => break,
            Lparen => ast::TokenKind::LeftBracket,
            Rparen => ast::TokenKind::RightBracket,
            Number => ast::TokenKind::Number(token_str.parse().unwrap()),
            Symbol => ast::TokenKind::Symbol(token_str.into()),
            // Skip whitespace for now
            Whitespace => continue,
        };

        result.push(ast::Token::with_span(
            kind,
            span.map(|s| ByteIndex(s as u32 + 1)),
        ));
    }

    result
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn tokenise_number_literals() {
        assert_eq!(
            vec![ast::Token::with_span(
                ast::TokenKind::Number(0),
                Span::new(ByteIndex(1), ByteIndex(2))
            )],
            tokenise("0")
        );
        assert_eq!(
            vec![ast::Token::with_span(
                ast::TokenKind::Number(1234),
                Span::new(ByteIndex(1), ByteIndex(5))
            )],
            tokenise("1234")
        );
    }
}