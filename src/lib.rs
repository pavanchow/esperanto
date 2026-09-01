//! Esperanto: a small statically-typed language you can read end to end.
//!
//! The pipeline is four readable stages: [`lexer`] turns text into tokens,
//! [`parser`] turns tokens into an AST, [`types`] checks and infers types over
//! that AST, and [`interpreter`] walks it. A program that does not type-check
//! never runs.
pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod types;
pub mod value;

pub use ast::Type;
pub use error::{Error, Result};
pub use value::Value;

/// Lex, parse, type-check, then evaluate `src`. Returns the value of the last
/// statement in the program.
pub fn run(src: &str) -> Result<Value> {
    let toks = lexer::lex(src)?;
    let prog = parser::parse(toks)?;
    types::check(&prog)?;
    interpreter::run_program(&prog)
}

/// Type-check `src` without running it. `Ok(())` means the program is well-typed.
pub fn typecheck(src: &str) -> Result<()> {
    let toks = lexer::lex(src)?;
    let prog = parser::parse(toks)?;
    types::check(&prog)
}
