use crate::{
    function::builtins::builtins,
    list,
    list::List,
    mapper_pool::{MapperPool, MapperPoolBuilder},
    Exception, Value, ValueKind,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

type ValueResult<'a> = Result<Arc<Value<'a>>, Arc<Value<'a>>>;

pub struct Context<'a> {
    pub pool: Arc<MapperPool<ValueResult<'a>>>,
    pub names: Mutex<HashMap<Cow<'a, str>, Arc<Value<'a>>>>,
    pub parent: Option<NonNull<Context<'a>>>,
    _pin: PhantomPinned,
}

// I'm scared
unsafe impl Send for Context<'_> {}
unsafe impl Sync for Context<'_> {}

impl<'a> Context<'a> {
    pub fn new() -> Pin<Box<Self>> {
        let pool = Arc::new(
            // FIXME: configurable pool
            MapperPoolBuilder::new()
                .pool_size(16)
                .storage_size(256)
                .build(),
        );
        Box::pin(Self {
            pool,
            names: Mutex::new(HashMap::new()),
            parent: None,
            _pin: PhantomPinned,
        })
    }

    pub fn fork(&self) -> Pin<Box<Self>> {
        Box::pin(Self {
            pool: self.pool.clone(),
            names: Mutex::new(self.names.lock().unwrap().clone()),
            parent: Some(self.into()),
            _pin: PhantomPinned,
        })
    }

    pub fn parent(&self) -> Option<&'a Self> {
        if let Some(parent) = self.parent {
            Some(unsafe { &*parent.as_ptr() })
        } else {
            None
        }
    }

    pub fn add_prelude(&self) {
        let mut names = self.names.lock().unwrap();
        for func in builtins() {
            names.insert(func.name.clone(), Value::simple(ValueKind::Function(func)));
        }
        names.insert(Cow::Borrowed("true"), Value::simple(ValueKind::Bool(true)));
        names.insert(
            Cow::Borrowed("false"),
            Value::simple(ValueKind::Bool(false)),
        );
    }

    pub fn find_ident(&self, ident: &str) -> Option<Arc<Value<'a>>> {
        let val = {
            let names = self.names.lock().unwrap();
            names.get(ident).map(Arc::clone)
        };
        if let Some(_) = &val {
            val
        } else if let Some(parent) = self.parent() {
            parent.find_ident(ident)
        } else {
            None
        }
    }

    fn try_evaluate(&self, val: Arc<Value<'a>>) -> ValueResult<'a> {
        let val = self.evaluate(val);
        if val.kind.is_exception() {
            Err(val)
        } else {
            Ok(val)
        }
    }

    fn evaluate_list(&self, list: &List<Value<'a>>, seq: bool) -> Arc<Value<'a>> {
        let new_values_res: Result<List<Value>, _> = if seq {
            list.iter().map(|val| self.try_evaluate(val)).collect()
        } else {
            self.pool.map(|val| self.try_evaluate(val), list.iter())
        };

        let new_values = match new_values_res {
            Ok(new_values) => new_values,
            Err(exc) => return exc,
        };

        let (func_val, args) = new_values.head_tail().expect("already checked length");

        if let ValueKind::Function(func) = &func_val.kind {
            (func.call)(self, args)
        } else {
            Arc::new(Value {
                raw: false,
                sequential: false,
                kind: ValueKind::Exception(Exception {
                    ident: "not-a-function",
                    args: List::new().cons_arc(func_val),
                }),
            })
        }
    }

    pub fn evaluate(&self, val: Arc<Value<'a>>) -> Arc<Value<'a>> {
        if val.raw {
            return val;
        }

        match val.kind {
            ValueKind::Symbol(ident) => match self.find_ident(ident) {
                Some(val) => val.clone(),
                None => Arc::new(Value {
                    raw: false,
                    sequential: false,
                    kind: ValueKind::Exception(Exception {
                        ident: "undefined-ident",
                        args: list![Value {
                            raw: true,
                            sequential: false,
                            kind: ValueKind::Symbol(ident),
                        }],
                    }),
                }),
            },
            ValueKind::List(ref list) if list.len() == 0 => val,
            ValueKind::List(ref list) => self.evaluate_list(list, val.sequential),
            _ => val,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{raw_value, sequential_value, simple_value};
    use crate::{list, Function};
    use std::ops::Deref;

    #[test]
    fn test_new() {
        assert_eq!(
            Context::new().names.lock().unwrap().deref(),
            &HashMap::new()
        );
    }

    #[test]
    fn test_fork() {
        let ctx1 = Context::new();
        ctx1.names
            .lock()
            .unwrap()
            .insert(Cow::Borrowed("key1"), Value::simple(ValueKind::Integer(1)));
        ctx1.names
            .lock()
            .unwrap()
            .insert(Cow::Borrowed("key2"), Value::simple(ValueKind::Float(3.14)));
        let ctx2 = ctx1.fork();
        assert_eq!(
            ctx1.names.lock().unwrap().deref(),
            ctx2.names.lock().unwrap().deref()
        );
    }

    fn simple_evaluate_test<'a>(ctx: &Context<'a>, val: ValueKind<'a>) {
        let simple_val = Value::simple(val.clone());
        let sequential_val = Arc::new(sequential_value(val.clone()));
        let raw_val = Arc::new(raw_value(val.clone()));
        assert_eq!(ctx.evaluate(simple_val.clone()), simple_val);
        assert_eq!(ctx.evaluate(sequential_val.clone()), sequential_val);
        assert_eq!(ctx.evaluate(raw_val.clone()), raw_val);
    }

    #[test]
    fn test_evaluate_scalar() {
        let ctx = Context::new();
        simple_evaluate_test(&ctx, ValueKind::Bool(true));
        simple_evaluate_test(&ctx, ValueKind::Integer(42));
        simple_evaluate_test(&ctx, ValueKind::Float(4.2));
        simple_evaluate_test(&ctx, ValueKind::String("test".into()));
        simple_evaluate_test(&ctx, ValueKind::List(list![]));
        simple_evaluate_test(
            &ctx,
            ValueKind::Function(Function {
                call: Arc::new(move |_ctx, val| Value::simple(ValueKind::List(val))),
                name: Cow::Borrowed("identity"),
            }),
        );
        simple_evaluate_test(
            &ctx,
            ValueKind::Exception(Exception {
                ident: "system-error",
                args: list![simple_value(ValueKind::Integer(0))],
            }),
        );
    }

    fn raw_evaluate_test<'a>(ctx: &Context<'a>, val: ValueKind<'a>) {
        let val = Arc::new(raw_value(val.clone()));
        assert_eq!(ctx.evaluate(val.clone()), val);
    }

    #[test]
    fn test_evaluate_raw() {
        let ctx = Context::new();
        raw_evaluate_test(&ctx, ValueKind::Symbol("test"));
        raw_evaluate_test(
            &ctx,
            ValueKind::List(list![
                simple_value(ValueKind::Symbol("test")),
                simple_value(ValueKind::Integer(42)),
            ]),
        );
    }

    #[test]
    fn test_evaluate_ident() {
        let ctx = Context::new();
        let key = Value::simple(ValueKind::Symbol("key"));
        let val = Value::simple(ValueKind::Integer(42));
        ctx.names
            .lock()
            .unwrap()
            .insert(Cow::Borrowed("key"), val.clone());
        assert_eq!(ctx.evaluate(key.clone()), val);
        // Check that value is not deleted from storage
        assert_eq!(ctx.evaluate(key.clone()), val);
    }

    #[test]
    fn test_undefined_ident() {
        let ctx = Context::new();
        assert_eq!(
            ctx.evaluate(Value::simple(ValueKind::Symbol("key"))),
            Value::simple(ValueKind::Exception(Exception {
                ident: "undefined-ident",
                args: list![raw_value(ValueKind::Symbol("key"))]
            }))
        )
    }
}
