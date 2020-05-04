use crate::{list::List, Value, ValueKind};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct Exception<'a> {
    pub ident: &'a str,
    pub args: List<Value<'a>>,
}

impl<'a> Exception<'a> {
    pub fn to_value(self) -> Arc<Value<'a>> {
        Value::simple(ValueKind::Exception(self))
    }

    pub fn wrong_type(expected: &'a str, val: Arc<Value<'a>>) -> Self {
        Self {
            ident: "wrong-type",
            args: List::new()
                .cons_arc(Value::simple(ValueKind::Symbol(expected)))
                .cons_arc(val),
        }
    }

    pub fn too_few_arguments(expected: i64) -> Self {
        Self {
            ident: "too-few-arguments",
            args: List::new().cons_arc(Value::simple(ValueKind::Integer(expected))),
        }
    }

    pub fn too_many_arguments(expected: i64) -> Self {
        Self {
            ident: "too-many-arguments",
            args: List::new().cons_arc(Value::simple(ValueKind::Integer(expected))),
        }
    }

    pub fn list_is_empty() -> Self {
        Self {
            ident: "list-is-empty",
            args: List::new(),
        }
    }
}
