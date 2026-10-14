//! bcalc: an interpreter for the expression subset of the POSIX `bc`
//! language. The reporting jobs used to shell out to `bc`; bcalc replaces it
//! in-process, so its output must be the same as bc's (see README.md).

mod interp;
pub mod json;
pub mod num;
mod parse;

pub use interp::run;
pub use num::Num;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Parse,
    Math,
    Runtime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn parse(m: &str) -> Error {
        Error { kind: ErrorKind::Parse, message: m.to_string() }
    }
    pub fn math(m: &str) -> Error {
        Error { kind: ErrorKind::Math, message: m.to_string() }
    }
    pub fn runtime(m: &str) -> Error {
        Error { kind: ErrorKind::Runtime, message: m.to_string() }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            ErrorKind::Parse => "Parse error",
            ErrorKind::Math => "Math error",
            ErrorKind::Runtime => "Runtime error",
        };
        write!(f, "{}: {}", kind, self.message)
    }
}

/// Result of running a program: everything printed to standard output, and
/// the error that stopped the program, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub output: String,
    pub error: Option<Error>,
}
