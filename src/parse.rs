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
    /// Single line comment
    Comment,
}

/// Tokenise a given string
///
/// Takes a given input string and transforms it into a vector of
/// tokens by running a state machine over it.
fn tokenise(source: &str) -> Vec<ast::Token> {
    use TokeniseState::*;

    let mut result = Vec::new();
    let mut start = 0;

    loop {
        let mut state = Start;
        let mut end = start;

        // Search through the remaining characters until the state
        // machine can make no further transitions.
        for c in source[start as usize..].chars() {
            // This two-level match encodes the state transitions for
            // the automaton. First we dispatch based on the current
            // state, then the character we are looking at.
            let next = match state {
                Start => match c {
                    '(' => Some(Lparen),
                    ')' => Some(Rparen),
                    '0'..='9' => Some(Number),
                    'a'..='z'
                    | 'A'..='Z'
                    | '!'
                    | '%'
                    | '&'
                    | '*'
                    | '+'
                    | '-'
                    | '.'
                    | '/'
                    | ':'
                    | '<'
                    | '='
                    | '>'
                    | '?'
                    | '@'
                    | '$'
                    | '^' => Some(Symbol),
                    ';' => Some(Comment),
                    c if c.is_whitespace() => Some(Whitespace),
                    _ => None,
                },
                Lparen | Rparen => None,
                Number => match c {
                    '0'..='9' => Some(Number),
                    _ => None,
                },
                Symbol => match c {
                    'A'..='Z'
                    | 'a'..='z'
                    | '!'
                    | '%'
                    | '&'
                    | '*'
                    | '+'
                    | '-'
                    | '.'
                    | '/'
                    | ':'
                    | '<'
                    | '='
                    | '>'
                    | '?'
                    | '@'
                    | '$'
                    | '^'
                    | '0'..='9' => Some(Symbol),
                    _ => None,
                },
                Whitespace => {
                    if c.is_whitespace() {
                        Some(Whitespace)
                    } else {
                        None
                    }
                }
                Comment => {
                    if c == '\r' || c == '\n' {
                        None
                    } else {
                        Some(Comment)
                    }
                }
            };

            // If we transitioned then accept the character by moving
            // on our `end` index.
            if let Some(next_state) = next {
                state = next_state;
                end += c.len_utf8();
            } else {
                break;
            }
        }

        let token_str = &source[start..end];
        let span = Span::new((start as u32) + 1, (end as u32) + 1);

        start = end;

        // all our states are accepting other than `Start` and
        // `Whitespace`. Choose the token kind based on the state we
        // have landed in.
        let kind = match state {
            // If no transition was followed from the start state we
            // have completed tokenisation
            Start => break,
            Lparen => ast::TokenKind::LeftBracket,
            Rparen => ast::TokenKind::RightBracket,
            Number => ast::TokenKind::Number(token_str.parse().unwrap()),
            Symbol => ast::TokenKind::Symbol(token_str.into()),
            // Skip whitespace for now
            Whitespace | Comment => continue,
        };

        result.push(ast::Token::with_span(kind, span));
    }

    result
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
                Number(n) => ast::Expr::Number(token, n),
                Symbol(ref s) => {
                    let sym = s.clone();
                    ast::Expr::Symbol(token, sym)
                }
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
                    let define_tok = self.0.next().unwrap();
                    let sym_tok = self.0.next().unwrap();
                    let value = self.parse_expr();
                    let close = self.0.next().unwrap();
                    ast::Expr::Define(open, define_tok, sym_tok, Box::new(value), close)
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
    ParseState(tokens.into_iter().peekable()).parse_expr()
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

    #[test]
    fn tokenise_symbols() {
        assert_eq!(
            vec![ast::Token::with_span(
                ast::TokenKind::Symbol("hello/world".into()),
                Span::new(ByteIndex(1), ByteIndex(12))
            )],
            tokenise("hello/world")
        );
        assert_eq!(
            vec![
                ast::Token::with_span(
                    ast::TokenKind::Symbol("hello".into()),
                    Span::new(ByteIndex(1), ByteIndex(6))
                ),
                ast::Token::with_span(
                    ast::TokenKind::Symbol("world".into()),
                    Span::new(ByteIndex(7), ByteIndex(12))
                )
            ],
            tokenise("hello world")
        );
        assert_eq!(
            vec![ast::Token::with_span(
                ast::TokenKind::Symbol("hello.world".into()),
                Span::new(ByteIndex(1), ByteIndex(12))
            )],
            tokenise("hello.world")
        );
        assert_eq!(
            vec![ast::Token::with_span(
                ast::TokenKind::Symbol("+".into()),
                Span::new(ByteIndex(1), ByteIndex(2))
            )],
            tokenise("+")
        )
    }

    #[test]
    fn tokenise_brackets() {
        assert_eq!(
            vec![ast::Token::with_span(
                ast::TokenKind::LeftBracket,
                Span::new(ByteIndex(1), ByteIndex(2))
            )],
            tokenise("(")
        );
        assert_eq!(
            vec![ast::Token::with_span(
                ast::TokenKind::RightBracket,
                Span::new(ByteIndex(1), ByteIndex(2))
            )],
            tokenise(")")
        );
        assert_eq!(
            vec![
                ast::Token::with_span(
                    ast::TokenKind::LeftBracket,
                    Span::new(ByteIndex(1), ByteIndex(2))
                ),
                ast::Token::with_span(
                    ast::TokenKind::RightBracket,
                    Span::new(ByteIndex(2), ByteIndex(3))
                )
            ],
            tokenise("()")
        );
        assert_eq!(
            vec![
                ast::Token::with_span(
                    ast::TokenKind::LeftBracket,
                    Span::new(ByteIndex(1), ByteIndex(2))
                ),
                ast::Token::with_span(
                    ast::TokenKind::LeftBracket,
                    Span::new(ByteIndex(2), ByteIndex(3))
                ),
                ast::Token::with_span(
                    ast::TokenKind::LeftBracket,
                    Span::new(ByteIndex(3), ByteIndex(4))
                ),
                ast::Token::with_span(
                    ast::TokenKind::RightBracket,
                    Span::new(ByteIndex(4), ByteIndex(5))
                ),
                ast::Token::with_span(
                    ast::TokenKind::RightBracket,
                    Span::new(ByteIndex(5), ByteIndex(6))
                ),
                ast::Token::with_span(
                    ast::TokenKind::RightBracket,
                    Span::new(ByteIndex(6), ByteIndex(7))
                )
            ],
            tokenise("((()))")
        );
    }

    #[test]
    fn tokenise_comments() {
        assert_eq!(Vec::<ast::Token>::new(), tokenise("; hello world"));
        assert_eq!(
            Vec::<ast::Token>::new(),
            tokenise("; hello world\n; another comment\r\n; windows eol")
        );
    }

    #[test]
    fn parse_atoms() {
        assert_eq!(
            ast::Expr::Number(
                ast::Token::with_span(
                    ast::TokenKind::Number(64),
                    Span::new(ByteIndex(1), ByteIndex(3))
                ),
                64
            ),
            parse("64")
        );
        assert_eq!(
            ast::Expr::Number(
                ast::Token::with_span(
                    ast::TokenKind::Number(12364),
                    Span::new(ByteIndex(1), ByteIndex(6))
                ),
                12364
            ),
            parse("12364")
        );
        assert_eq!(
            ast::Expr::Number(
                ast::Token::with_span(
                    ast::TokenKind::Number(9223372036854775807),
                    Span::new(ByteIndex(1), ByteIndex(20))
                ),
                9223372036854775807
            ),
            parse("9223372036854775807")
        );
    }
}
