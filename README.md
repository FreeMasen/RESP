[![travis](https://img.shields.io/travis/FreeMasen/RESSA.svg)](https://travis-ci.org/FreeMasen/RESSA)
[![crates.io](https://img.shields.io/crates/v/ressa.svg)](https://crates.io/crates/ressa)
[![last commit master](https://img.shields.io/github/last-commit/FreeMasen/RESSA.svg)](https://github.com/FreeMasen/RESSA/commits/master)
# RESSA
> Rust EcmaScript Syntax Analyzer

This project is part of a series of crates designed to enable developers to create JavaScript development tools using the Rust programming language. [Rusty ECMA Details](#rusty-ecma-details)

The two major pieces that users will interact with are the `Parser` struct and the enums defined by `resast`

## `Parser`

The parser struct will be the main way to convert text into an `AST`.
Conveniently `Parser` implements `Iterator` over `Result<ProgramPart, Error>`,
this means that you can evaluate your JS in pieces from top to bottom. These pieces will be discussed in more detail in the [node section](#node).

### Iterator Example
```rust
use ressa::Parser;
use resast::prelude::*;
fn main() {
    let js = "function helloWorld() { alert('Hello world'); }";
    let p = Parser::new(&js).unwrap();
    let f = ProgramPart::decl(
        Decl::Func(
            Function {
                id: Some("helloWorld".to_string()),
                params: vec![],
                body: vec![
                    ProgramPart::Stmt(
                        Stmt::Expr(
                            Expr::call(Expr::ident("alert"), vec![Expr::string("'Hello world'")])
                        )
                    )
                ],
                generator: false,
                is_async: false,
            }
        )
    );
    for part in p {
        assert_eq!(part.unwrap(), f);
    }
}
```

Another way to interact with a `Parser` would be to utilize the `parse` method. This method will iterate over all of the found `ProgramParts` and collect them into a `Program`,

### Parse Example
```rust
use ressa::{
    Parser,
};
use resast::prelude::*;
fn main() {
    let js = "
function Thing() {
    return 'stuff';
}
";
    let mut parser = Parser::new(js).expect("Failed to create parser");
    let program = parser.parse().expect("Unable to parse text");
    match program {
        Program::Script(parts) => println!("found a script"),
        Program::Mod(parts) => println!("found an es6 module"),
    }
}
```
Once you get to the inner `parts` of a `Program` you have a `Vec<ProgramPart>` which will operate the same as the [iterator example](#iterator-example)

# Rusty ECMA Details
## The Rust ECMA Crates
- [RESS](https://github.com/freemasen/ress) - Tokenizer or Scanner
- [RESSA](https://github.com/freemasen/ressa) - Parser
- [RESAST](https://github.com/freemasen/resast) - AST
- [RESW](https://github.com/freemasen/resw) - Writer

## Why So Many?
While much of what each crate provides is closely coupled with the other crates, the main goal is to provide the largest amount of customizability. For example, someone writing a fuzzer would only need the `RESAST` and `RESW`, it seems silly to require that they also pull in `RESS` and `RESSA` needlessly.

