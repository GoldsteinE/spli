pub mod executor;
pub mod list;
pub mod parser;

pub mod function;
pub use function::Function;

mod exception;
pub use exception::Exception;

pub mod mapper_pool;

#[cfg(test)]
pub mod test_helpers;

use list::List;
use std::{
    fmt::{self, Write},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind<'a> {
    Symbol(&'a str),
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(List<Value<'a>>),
    Function(Function<'a>),
    Exception(Exception<'a>),
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
            Self::Bool(b) => write!(fmt, "{}", b),
            Self::Integer(n) => write!(fmt, "{}", n),
            Self::Float(x) => write!(fmt, "{}", x),
            Self::String(s) => write!(fmt, "{:?}", s),
            Self::List(xs) => write!(fmt, "{}", xs),
            Self::Function(func) => write!(fmt, "{:?}", func),
            Self::Exception(exc) => write!(fmt, "{{exception {}}}", exc.ident),
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
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::List(_) => "list",
            Self::Function(_) => "function",
            Self::Exception(_) => "exception",
        }
    }

    pub fn is_exception(&self) -> bool {
        if let Self::Exception(_) = self {
            true
        } else {
            false
        }
    }
}

impl<'a> Value<'a> {
    pub fn simple(kind: ValueKind<'a>) -> Arc<Self> {
        Arc::new(Self {
            kind,
            raw: false,
            sequential: false,
        })
    }

    pub fn raw(kind: ValueKind<'a>) -> Arc<Self> {
        Arc::new(Self {
            kind,
            raw: true,
            sequential: false,
        })
    }

    pub fn sequential(kind: ValueKind<'a>) -> Arc<Self> {
        Arc::new(Self {
            kind,
            raw: false,
            sequential: true,
        })
    }

    pub fn unit() -> Arc<Self> {
        Self::simple(ValueKind::List(List::new()))
    }
}
