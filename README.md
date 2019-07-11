# 🏎 Formula One - An Experimental LISP 🏎

Formula One is an experiment in ways to ergonomically build syntax trees and transformations in Rust.

```
🏎  > (begin (define foo 1007) (define bar 330) (+ foo bar))
 ~> 1337
```

## Blog Post

The early development of this language is discussed on my blog in [Lisp in Two Days with Rust][blogpost]

 [blogpost]: https://willspeak.me/2019/07/10/lisp-in-two-days-with-rust.html
 
## Features

The language is a small subset of the LISP described in <https://norvig.com/lispy.html>. Notably it supports the following special forms:

 * `(if <cond> <then> <elze>)` for conditional evaluation of `<then>` or `<elze>`
 * `(define <sym> <expr>)` binding a value to a symbol
 * `(<sym> <args>...)` for calling a named function `<sym>`

All evaluation takes place in a single global environment. The language does not support user-defined functions with `labda` or the nested environments that they would entail. Quoting of values with `'` or `quote` is also not supported. There is no comment support yet either.

## 🐉 Here be Dragons 🐉

This is only intended as an experiment to develop techniques for building syntax trees in code. It isn't intended as a production use language.
