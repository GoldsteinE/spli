pub mod list;
pub mod parser;

use std::fmt::{self, Write};
use list::List;

#[derive(Debug, PartialEq)]
pub enum ValueKind<'a> {
    Symbol(&'a str),
    Integer(i64),
    Float(f64),
    String(String),
    List(List<Value<'a>>),
}

#[derive(Debug, PartialEq)]
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
