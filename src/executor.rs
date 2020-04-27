use crate::{Value, ValueKind};
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error<'a> {
    #[error("undefined ident: {0}")]
    UndefinedIdent(&'a str),
}

pub type Result<'a, T, E = Error<'a>> = std::result::Result<T, E>;

pub struct Context<'a> {
    pool: Arc<ThreadPool>,
    pub names: HashMap<Cow<'a, str>, Arc<Value<'a>>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(ThreadPoolBuilder::new().build().unwrap()),
            names: HashMap::new(),
        }
    }

    pub fn fork(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            names: self.names.clone(),
        }
    }

    pub fn evaluate(&self, val: Arc<Value<'a>>) -> Result<Arc<Value<'a>>> {
        if val.raw {
            return Ok(val);
        }

        match val.kind {
            ValueKind::Float(_) | ValueKind::Integer(_) | ValueKind::String(_) | ValueKind::Function(_) => Ok(val),
            ValueKind::Symbol(ident) => match self.names.get(ident) {
                Some(val) => Ok(val.clone()),
                None => Err(Error::UndefinedIdent(ident)),
            },
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{list, Function};
    use crate::test_helpers::{raw_value, sequential_value, simple_value};

    #[test]
    fn test_new() {
        assert_eq!(Context::new().names, HashMap::new());
    }

    #[test]
    fn test_fork() {
        let mut ctx1 = Context::new();
        ctx1.names.insert(
            Cow::Borrowed("key1"),
            Arc::new(simple_value(ValueKind::Integer(1))),
        );
        ctx1.names.insert(
            Cow::Borrowed("key2"),
            Arc::new(simple_value(ValueKind::Float(3.14))),
        );
        let ctx2 = ctx1.fork();
        assert_eq!(ctx1.names, ctx2.names);
    }

    fn simple_evaluate_test<'a, 'b>(ctx: &Context<'a>, val: ValueKind<'b>) {
        let simple_val = Arc::new(simple_value(val));
        /*
        let sequential_val = Arc::new(sequential_value(val.clone()));
        let raw_val = Arc::new(raw_value(val));
        */
        ctx.evaluate(simple_val);
        /*
        assert_eq!(ctx.evaluate(simple_val.clone()).unwrap(), simple_val);
        assert_eq!(
            ctx.evaluate(sequential_val.clone()).unwrap(),
            sequential_val
        );
        assert_eq!(ctx.evaluate(raw_val.clone()).unwrap(), raw_val);
        */
    }

    #[test]
    fn test_evaluate_scalar() {
        let ctx = Context::new();
        simple_evaluate_test(&ctx, ValueKind::Integer(42));
        simple_evaluate_test(&ctx, ValueKind::Float(4.2));
        simple_evaluate_test(&ctx, ValueKind::String("test".into()));
        /* simple_evaluate_test(&ctx, ValueKind::Function(Function {
            call: Arc::new(move |val| simple_value(ValueKind::List(val))),
            name: Cow::Borrowed("identity"),
        }));
        */
    }

    /*
    fn raw_evaluate_test<'a>(ctx: &'a Context<'a>, val: ValueKind<'a>) {
        let val = Arc::new(raw_value(val.clone()));
        assert_eq!(ctx.evaluate(val.clone()).unwrap(), val);
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
    */

    #[test]
    fn test_evaluate_ident() {
        let mut ctx = Context::new();
        let key = Arc::new(simple_value(ValueKind::Symbol("key")));
        let val = Arc::new(simple_value(ValueKind::Integer(42)));
        ctx.names.insert(Cow::Borrowed("key"), val.clone());
        assert_eq!(ctx.evaluate(key.clone()).unwrap(), val);
        // Check that value is not deleted from storage
        assert_eq!(ctx.evaluate(key.clone()).unwrap(), val);
    }
}
