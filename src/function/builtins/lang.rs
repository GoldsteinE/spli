use super::parse_args;
use crate::Function;
use crate::{exception::Exception, executor::Context, list::List, Value, ValueKind};
use std::borrow::Cow;
use std::sync::Arc;

pub fn def<'a>(ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, {
        let key = ValueKind::Symbol | "ident",
        let val
    });
    ctx.names
        .lock()
        .unwrap()
        .insert(Cow::Borrowed(key), val.clone());
    val
}

fn create_function<'a>(
    ctx: &Context<'a>,
    name: &'a str,
    mut args: List<Value<'a>>,
) -> Arc<Value<'a>> {
    parse_args!(args, {
        let fn_args = ValueKind::List | "list",
        let fn_body
    });

    let idents: Result<Vec<_>, _> = fn_args
        .iter()
        .map(|val| {
            if let ValueKind::Symbol(ident) = val.kind {
                Ok(ident)
            } else {
                Err(val)
            }
        })
        .collect();

    let idents = match idents {
        Ok(idents) => idents,
        Err(val) => return Exception::wrong_type("ident", val).to_value(),
    };

    let fn_body = Value::simple(fn_body.kind.clone());

    // FIXME: Can we avoid forking twice?
    let ctx = ctx.fork();
    let fn_args = fn_args.clone();
    Value::simple(ValueKind::Function(Function {
        name: Cow::Borrowed(name),
        call: Arc::new(move |_caller_ctx, args| {
            let ctx = ctx.fork();

            let needed_len = fn_args.len();
            let actual_len = args.len();
            if needed_len < actual_len {
                return Exception::too_many_arguments(needed_len as i64).to_value();
            }
            if needed_len > actual_len {
                return Exception::too_few_arguments(needed_len as i64).to_value();
            }

            {
                let mut names = ctx.names.lock().unwrap();
                for (argname, arg) in idents.iter().zip(args) {
                    names.insert(Cow::Borrowed(*argname), arg);
                }
            }

            ctx.evaluate(fn_body.clone())
        }),
    }))
}

pub fn defn<'a>(ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    let name = match args.pop() {
        Some(val) => match val.kind {
            ValueKind::Symbol(ident) => ident,
            _ => return Exception::wrong_type("ident", val).to_value(),
        },
        None => return Exception::too_few_arguments(3).to_value(),
    };

    let func = create_function(ctx, name, args);
    ctx.names
        .lock()
        .unwrap()
        .insert(Cow::Borrowed(name), func.clone());
    func
}

pub fn func_literal<'a>(ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    create_function(ctx, "<lambda>", args)
}

pub fn create_list<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    Value::simple(ValueKind::List(args))
}

pub fn do_return<'a>(_ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    let mut res = Value::simple(ValueKind::List(List::new()));
    while let Some(item) = args.pop() {
        res = item;
    }
    res
}

pub fn do_discard<'a>(_ctx: &Context<'a>, _args: List<Value<'a>>) -> Arc<Value<'a>> {
    Value::unit()
}

pub fn if_func<'a>(ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, {
        let pred = ValueKind::Bool | "bool",
        let if_true,
        let @mut rest
    });

    let if_false = if rest.len() != 0 {
        parse_args!(rest, { let if_false });
        if_false
    } else {
        Value::unit()
    };

    if *pred {
        ctx.evaluate(Value::simple(if_true.kind.clone()))
    } else {
        ctx.evaluate(Value::simple(if_false.kind.clone()))
    }
}

fn truish(val: &Value<'_>) -> bool {
    match &val.kind {
        ValueKind::Bool(false) => false,
        ValueKind::List(list) => list.len() != 0,
        _ => true
    }
}

pub fn atom<'a>(_ctx: &Context<'a>, mut args: List<Value<'a>>) -> Arc<Value<'a>> {
    parse_args!(args, { let arg });
    if let ValueKind::List(list) = &arg.kind {
        Value::simple(ValueKind::Bool(list.len() == 0))
    } else {
        Value::simple(ValueKind::Bool(true))
    }
}

pub fn cond<'a>(_ctx: &Context<'a>, args: List<Value<'a>>) -> Arc<Value<'a>> {
    for branch in args {
        if let ValueKind::List(list) = &branch.kind {
            if let Some((head, tail)) = list.head_tail() {
                if truish(&head) {
                    if tail.len() == 0 {
                        return head;
                    } else if let Some(last) = tail.last() {
                        return last;
                    }
                }
            }
        } else if truish(&branch) {
            return branch
        }
    }
    Value::unit()
}

pub(crate) fn builtins<'a>() -> Vec<Function<'a>> {
    vec![
        Function {
            name: Cow::Borrowed("def"),
            call: Arc::new(def),
        },
        Function {
            name: Cow::Borrowed("defn"),
            call: Arc::new(defn),
        },
        Function {
            name: Cow::Borrowed("fn"),
            call: Arc::new(func_literal),
        },
        Function {
            name: Cow::Borrowed("list"),
            call: Arc::new(create_list),
        },
        Function {
            name: Cow::Borrowed("do"),
            call: Arc::new(do_return),
        },
        Function {
            name: Cow::Borrowed("do_"),
            call: Arc::new(do_discard),
        },
        Function {
            name: Cow::Borrowed("atom"),
            call: Arc::new(atom),
        },
        Function {
            name: Cow::Borrowed("cond"),
            call: Arc::new(cond),
        },
    ]
}
