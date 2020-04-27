pub mod executor;
pub mod list;
pub mod parser;

mod function;
pub use function::Function;

#[cfg(test)]
pub mod test_helpers;

use list::List;
use std::fmt::{self, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind<'a> {
    Symbol(&'a str),
    Integer(i64),
    Float(f64),
    String(String),
    List(List<Value<'a>>),
    Function(Function<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value<'a> {
    pub raw: bool,
    pub sequential: bool,
    pub kind: ValueKind<'a>,
}

impl fmt::Display for ValueKind<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbol(s) => fmt.write_str(s),
            Self::Integer(n) => write!(fmt, "{}", n),
            Self::Float(x) => write!(fmt, "{}", x),
            Self::String(s) => write!(fmt, "{:?}", s),
            Self::List(xs) => write!(fmt, "{}", xs),
            Self::Function(func) => write!(fmt, "{:?}", func),
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw {
            fmt.write_char('\'')?;
        }
        if self.sequential {
            fmt.write_char('!')?;
        }
        write!(fmt, "{}", self.kind)
    }
}

impl ValueKind<'_> {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Symbol(_) => "symbol",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::List(_) => "list",
            Self::Function(_) => "function",
        }
    }
}
