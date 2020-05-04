use super::parse_args;
use crate::{
    exception::Exception, executor::Context, function::Function, list::List, Value, ValueKind,
};
use std::{
    borrow::Cow,
    sync::Arc,
    time::{Duration, SystemTime},
};

pub fn debug<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    args.into_iter().for_each(|val| println!("{}", val));

    Value::unit()
}

pub fn sleep<'a>(_ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, { let time });

    // FIXME: exception on negatives
    let time = match time.kind {
        ValueKind::Integer(num) => Duration::from_secs(num as u64),
        ValueKind::Float(num) => Duration::from_secs(num as u64),
        _ => return Exception::wrong_type("number", time).to_value(),
    };

    std::thread::sleep(time);

    Value::unit()
}

pub fn time<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, {});
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or_else(|e| -e.duration().as_secs_f64());
    Value::simple(ValueKind::Float(now))
}

pub(crate) fn builtins<'a>() -> Vec<Function<'a>> {
    vec![
        Function {
            name: Cow::Borrowed("debug"),
            call: Arc::new(debug),
        },
        Function {
            name: Cow::Borrowed("sleep"),
            call: Arc::new(sleep),
        },
        Function {
            name: Cow::Borrowed("time"),
            call: Arc::new(time),
        },
    ]
}
