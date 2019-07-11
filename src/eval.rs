//! Execution and Evaluation
//!
//! This module is responsible for walking expression trees and
//! evaluating the programs that they represent. It revolves around
//! the `eval` method.

use super::ast;

use std::collections::HashMap;
use std::fmt;

/// Stores one of the varying value kinds that are used in
/// evaluation. This can be the result of evaluating an expression or
/// stored in an environment.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Value {
    /// A numeric value
    Number(i64),
    /// A callable value
    Callable(Callable),
    /// The empty list and an invalid or placeholder value
    Nil,
}

impl Value {
    /// Check the trunthyness of a given value
    fn is_truthy(&self) -> bool {
        use Value::*;
        match *self {
            Number(n) => n != 0,
            _ => true,
        }
    }

    /// Convert a value to a number
    fn into_num(self) -> i64 {
        match self {
            Value::Number(n) => n,
            other => panic!("can't use {:?}, it isn't a number", other),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Number(n) => write!(out, "{}", n),
            Value::Callable(c) => write!(out, "<callable {:x?}>", c),
            Value::Nil => write!(out, "nil"),
        }
    }
}

/// Evaluation error values
///
/// This contains the different kinds of errors that can occur when
/// evaluating a value.
#[derive(Debug, PartialEq)]
pub struct EvalError(String);

impl fmt::Display for EvalError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "error: {}", self.0)
    }
}

/// Evaluation Result Type
///
/// Used as the return value of calls to `eval`. Returns a `Value` on
/// success or an `EvalError` on failure.
pub type EvalResult = Result<Value, EvalError>;

/// The type of a funtion call in our LISP
type Callable = fn(Vec<Value>) -> EvalResult;

/// Simple Evaluation
///
/// Convenience function to evaluate a given expression in a new
/// environment. This is used by the main driver when evaluating
/// expressions from a function.
pub fn eval(expr: ast::Expr) -> EvalResult {
    eval_with_env(expr, &mut make_global_env())
}

/// Main evaluation function. This function accepts a parsed syntax
/// tree and evaluates it into a single Value using the given
/// environment..
pub fn eval_with_env(expr: ast::Expr, env: &mut HashMap<String, Value>) -> EvalResult {
    use ast::Expr::*;
    match expr {
        Symbol(_, s) => env
            .get(&s)
            .cloned()
            .ok_or_else(|| EvalError(format!("eval: Undefined symbol {}", s))),
        Number(_, n) => Ok(Value::Number(n)),
        If(_, _, cond, then, elz, _) => Ok(if eval_with_env(*cond, env)?.is_truthy() {
            eval_with_env(*then, env)?
        } else {
            eval_with_env(*elz, env)?
        }),
        Define(_, _, sym, value, _) => {
            let value = eval_with_env(*value, env)?;
            let sym = to_sym(sym)?;
            env.insert(sym, value.clone());
            Ok(value)
        }
        Call(_, sym, args, _) => {
            let sym = to_sym(sym)?;
            match env.get(&sym) {
                Some(Value::Callable(c)) => c(args
                    .into_iter()
                    .map(|a| eval_with_env(a, env))
                    .collect::<Result<Vec<_>, _>>()?),
                _ => Err(EvalError(format!("eval: Invalid function {}", sym))),
            }
        }
    }
}

/// Convert a token to a symbol.
fn to_sym(token: ast::Token) -> Result<String, EvalError> {
    match token.kind {
        ast::TokenKind::Symbol(s) => Ok(s),
        other => Err(EvalError(format!("Token '{:?}' is not symbol", other))),
    }
}

/// Get the last value or `Nil` if there are none
fn last_or_nil(values: Vec<Value>) -> Value {
    values.last().cloned().unwrap_or(Value::Nil)
}

/// Create the global environment. This is the root environment and
/// has the builtin operators and functions defined in it.
pub fn make_global_env() -> HashMap<String, Value> {
    let mut env = HashMap::new();

    env.insert(
        "print".into(),
        Value::Callable(|values| {
            for value in values.iter() {
                println!("{}", value);
            }
            Ok(last_or_nil(values))
        }),
    );
    env.insert(
        "exit".into(),
        Value::Callable(|values| {
            let status = values.into_iter().last().unwrap_or(Value::Number(0));
            std::process::exit(status.into_num() as i32)
        }),
    );
    env.insert(
        "begin".into(),
        Value::Callable(|values| Ok(last_or_nil(values))),
    );
    env.insert(
        "+".into(),
        Value::Callable(|values| Ok(Value::Number(values.iter().map(|i| i.into_num()).sum()))),
    );
    env.insert(
        "*".into(),
        Value::Callable(|values| Ok(Value::Number(values.iter().map(|i| i.into_num()).product()))),
    );
    env.insert(
        "-".into(),
        Value::Callable(|values| {
            Ok(if let Some((first, rest)) = values.split_first() {
                let first = first.into_num();
                if rest.len() == 0 {
                    Value::Number(-first)
                } else {
                    Value::Number(rest.iter().fold(first, |acc, n| acc - n.into_num()))
                }
            } else {
                // (-) ~> 0 ; apparently
                Value::Number(0)
            })
        }),
    );
    env.insert(
        "/".into(),
        Value::Callable(|values| {
            if let Some((first, rest)) = values.split_first() {
                let first = first.into_num();
                Ok(if rest.len() == 0 {
                    Value::Number(1 / first)
                } else {
                    Value::Number(rest.iter().fold(first, |acc, n| acc / n.into_num()))
                })
            } else {
                Err(EvalError("Wrong number of arguments: /, 0".into()))
            }
        }),
    );

    env
}
