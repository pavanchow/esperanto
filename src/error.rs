//! One error type for every stage, each carrying enough to point at the problem.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Lex { msg: String, line: usize },
    Parse { msg: String, line: usize },
    Type(String),
    Runtime(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lex { msg, line } => write!(f, "lex error (line {line}): {msg}"),
            Error::Parse { msg, line } => write!(f, "parse error (line {line}): {msg}"),
            Error::Type(msg) => write!(f, "type error: {msg}"),
            Error::Runtime(msg) => write!(f, "runtime error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
