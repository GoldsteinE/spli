use super::{parse_args, Function};
use crate::{exception::Exception, executor::Context, list::List, Value, ValueKind};
use std::borrow::Cow;
use std::sync::Arc;

pub fn cons<'a>(_ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, {
        let left,
        let right
    });
    if let ValueKind::List(list) = &right.kind {
        Value::simple(ValueKind::List(list.cons_arc(left)))
    } else {
        Value::simple(ValueKind::List(List::new().cons_arc(right).cons_arc(left)))
    }
}

pub fn head<'a>(_ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, { let list = ValueKind::List | "list" });
    if let Some(head) = list.head() {
        head
    } else {
        Exception::list_is_empty().to_value()
    }
}

pub fn tail<'a>(_ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, { let list = ValueKind::List | "list" });
    if let Some(tail) = list.tail() {
        Value::simple(ValueKind::List(tail))
    } else {
        Exception::list_is_empty().to_value()
    }
}

pub fn builtins<'a>() -> Vec<Function<'a>> {
    vec![
        Function {
            name: Cow::Borrowed("cons"),
            call: Arc::new(cons),
        },
        Function {
            name: Cow::Borrowed("head"),
            call: Arc::new(head),
        },
        Function {
            name: Cow::Borrowed("tail"),
            call: Arc::new(tail),
        },
    ]
}
