use super::parse_args;
use crate::{
    exception::Exception, executor::Context, function::Function, list::List, Value, ValueKind,
};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::sync::Arc;

fn commutative_numeric<'a, I, F>(
    args: List<Value<'a>>,
    start: i64,
    op_i: I,
    op_f: F,
) -> Arc<Value<'a>>
where
    I: Fn(i64, i64) -> i64,
    F: Fn(f64, f64) -> f64,
{
    use ValueKind::{Float, Integer};

    let mut accum = Integer(start);
    for val in args {
        accum = match (&accum, &val.kind) {
            (&Integer(a), &Integer(v)) => Integer(op_i(a, v)),
            (&Integer(a), &Float(v)) => Float(op_f(a as f64, v)),
            (&Float(a), &Integer(v)) => Float(op_f(a, v as f64)),
            (_, _) => return Exception::wrong_type("number", val).to_value(),
        }
    }
    Arc::new(Value {
        raw: false,
        sequential: false,
        kind: accum,
    })
}

pub fn add<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    commutative_numeric(args, 0, |a, b| a + b, |a, b| a + b)
}

pub fn mul<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    commutative_numeric(args, 1, |a, b| a * b, |a, b| a * b)
}

fn apply_to_numbers<'a, T>(
    left: &Arc<Value<'a>>,
    right: &Arc<Value<'a>>,
    apply_ints: impl Fn(&i64, &i64) -> T,
    apply_floats: impl Fn(&f64, &f64) -> T,
) -> Result<T, Arc<Value<'a>>> {
    use ValueKind::{Float, Integer};

    match (&left.kind, &right.kind) {
        (Integer(left), Integer(right)) => Ok(apply_ints(left, right)),
        (Integer(left), Float(right)) => Ok(apply_floats(&(*left as f64), right)),
        (Float(left), Integer(right)) => Ok(apply_floats(left, &(*right as f64))),
        (Float(left), Float(right)) => Ok(apply_floats(left, right)),
        (Float(_), _) | (Integer(_), _) => {
            Err(Exception::wrong_type("number", right.clone()).to_value())
        }
        _ => Err(Exception::wrong_type("number", left.clone()).to_value()),
    }
}

fn noncommutative_numeric<'a, I, F>(mut args: List<Value<'a>>, op_i: I, op_f: F) -> Arc<Value<'a>>
where
    I: Fn(&i64, &i64) -> i64,
    F: Fn(&f64, &f64) -> f64,
{
    parse_args!(args, { let left, let right });
    apply_to_numbers(
        &left,
        &right,
        |a, b| Value::simple(ValueKind::Integer(op_i(a, b))),
        |a, b| Value::simple(ValueKind::Float(op_f(a, b))),
    )
    .unwrap_or_else(|e| e)
}

pub fn sub<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    noncommutative_numeric(args, |a, b| a - b, |a, b| a - b)
}

// FIXME: division by zero
pub fn div<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    noncommutative_numeric(args, |a, b| a / b, |a, b| a / b)
}

pub fn transitive_compare<'a>(
    args: List<Value<'a>>,
    default: bool,
    comp: impl Fn(&Arc<Value<'a>>, &Arc<Value<'a>>) -> Result<bool, Arc<Value<'a>>>,
) -> Arc<Value<'a>> {
    let mut it = args.iter();
    if let Some(first) = it.next() {
        let mut acc = default;
        let mut prev = first;
        for val in it {
            let step = match comp(&prev, &val) {
                Ok(b) => b,
                Err(exc) => return exc,
            };
            if default {
                acc = acc && step;
            } else {
                acc = acc || step;
            }
            prev = val;
        }
        Value::simple(ValueKind::Bool(acc))
    } else {
        Value::simple(ValueKind::Bool(default))
    }
}

pub fn transitive_numeric_compare<'a>(
    args: List<Value<'a>>,
    default: bool,
    comp_ints: impl Fn(&i64, &i64) -> bool,
    comp_floats: impl Fn(&f64, &f64) -> bool,
) -> Arc<Value<'a>> {
    transitive_compare(args, default, |x, y| {
        apply_to_numbers(x, y, &comp_ints, &comp_floats)
    })
}

pub fn eq<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    transitive_compare(args, false, |x, y| Ok(x == y))
}

pub fn ne<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    transitive_compare(args, false, |x, y| Ok(x != y))
}

pub fn lt<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    transitive_numeric_compare(args, true, |x, y| x < y, |x, y| x < y)
}

pub fn le<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    transitive_numeric_compare(args, true, |x, y| x <= y, |x, y| x <= y)
}

pub fn gt<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    transitive_numeric_compare(args, true, |x, y| x > y, |x, y| x > y)
}

pub fn ge<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    transitive_numeric_compare(args, true, |x, y| x >= y, |x, y| x >= y)
}

pub fn cmp<'a>(_ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, { let left, let right });

    let token = match apply_to_numbers(&left, &right, i64::partial_cmp, f64::partial_cmp) {
        Ok(Some(Ordering::Less)) => "less",
        Ok(Some(Ordering::Equal)) => "equal",
        Ok(Some(Ordering::Greater)) => "greater",
        Ok(None) => "uncomparable",
        Err(exc) => return exc,
    };

    Value::raw(ValueKind::Symbol(token))
}

pub(crate) fn builtins<'a>() -> Vec<Function<'a>> {
    vec![
        Function {
            name: Cow::Borrowed("+"),
            call: Arc::new(add),
        },
        Function {
            name: Cow::Borrowed("*"),
            call: Arc::new(mul),
        },
        Function {
            name: Cow::Borrowed("-"),
            call: Arc::new(sub),
        },
        Function {
            name: Cow::Borrowed("/"),
            call: Arc::new(div),
        },
        Function {
            name: Cow::Borrowed("=="),
            call: Arc::new(eq),
        },
        Function {
            name: Cow::Borrowed("/="),
            call: Arc::new(ne),
        },
        Function {
            name: Cow::Borrowed("<"),
            call: Arc::new(lt),
        },
        Function {
            name: Cow::Borrowed("<="),
            call: Arc::new(le),
        },
        Function {
            name: Cow::Borrowed(">"),
            call: Arc::new(gt),
        },
        Function {
            name: Cow::Borrowed(">="),
            call: Arc::new(ge),
        },
        Function {
            name: Cow::Borrowed("cmp"),
            call: Arc::new(cmp),
        },
    ]
}
