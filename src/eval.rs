//! Execution and Evaluation
//!
//! This module is responsible for walking expression trees and
//! evaluating the programs that they represent. It revolves around
//! the `eval` method.

use super::ast;

use std::fmt;
use std::collections::HashMap;

/// Stores one of the varying value kinds that are used in
/// evaluation. This can be the result of evaluating an expression or
/// stored in an environment.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Value {
    /// A numeric value
    Number(i64),
    /// A callable value
    Callable(Callable),
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
        }
    }
}

/// The type of a funtion call in our LISP
type Callable = fn(Vec<Value>) -> Value;

/// Main evaluation function. This function accepts a parsed syntax
/// tree and evaluates it into a single Value.
pub fn eval(expr: ast::Expr) -> Value {
    let mut env = make_env();
    eval_with_env(expr, &mut env)
}

/// Evaluate with a given environment
fn eval_with_env(expr: ast::Expr, env: &mut HashMap<String, Value>) -> Value {
    use ast::Expr::*;
    match expr {
        Symbol(_, s) => env[&s],
        Number(_, n) => Value::Number(n),
        If(_, _, cond, then, elz, _) => {
            if eval_with_env(*cond, env).is_truthy() {
                eval_with_env(*then, env)
            } else {
                eval_with_env(*elz, env)
            }
        }
        Define(_, _, sym, value, _) => {
            let value = eval_with_env(*value, env);
            let sym = match sym.kind {
                ast::TokenKind::Symbol(s) => s,
                other => panic!("can't define '{:?}', it isn't a symbol", other),
            };
            env.insert(sym, value.clone());
            value
        }
        Call(_, sym, args, _) => {
            let sym = match sym.kind {
                ast::TokenKind::Symbol(s) => s,
                other => panic!("can't call '{:?}', it isn't a symbol", other),
            };
            match env.get(&sym) {
                Some(Value::Callable(c)) => {
                    c(args.into_iter().map(|a| eval_with_env(a, env)).collect())
                }
                _ => panic!("{:?} is not callable", sym),
            }
        }
    }
}

fn make_env() -> HashMap<String, Value> {
    let mut env = HashMap::new();

    env.insert(
        "print".into(),
        Value::Callable(|values| {
            for value in values.iter() {
                println!("{:?}", value);
            }
            values.last().cloned().unwrap_or(Value::Number(0))
        }),
    );
    env.insert(
        "exit".into(),
        Value::Callable(|values| {
            let status = values.into_iter().last().unwrap_or(Value::Number(0));
            std::process::exit(status.into_num() as i32);
        })
    );
    env.insert(
        "begin".into(),
        Value::Callable(|values| {
            values.into_iter().last().unwrap_or(Value::Number(0))
        })
    );
    env.insert(
        "+".into(),
        Value::Callable(|values| {
            let mut sum = 0;
            for value in values.iter() {
                sum += value.into_num();
            }
            Value::Number(sum)
        }),
    );
    env.insert(
        "-".into(),
        Value::Callable(|values| {
            if let Some((first, rest)) = values.split_first() {
                let mut sum = first.into_num();
                if rest.len() == 0 {
                    Value::Number(-sum)
                } else {
                    for value in rest {
                        sum -= value.into_num();
                    }
                    Value::Number(sum)
                }
            } else {
                panic!("No arguments to '-'")
            }
        }),
    );

    env
}
