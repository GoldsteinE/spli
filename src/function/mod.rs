pub mod builtins;

use super::{executor::Context, list::List, Value};
use std::{
    borrow::Cow,
    fmt::{self, Debug},
    sync::Arc,
};

#[derive(Clone)]
pub struct Function<'a> {
    pub call: Arc<dyn Fn(&Context<'a>, List<Value<'a>>) -> Arc<Value<'a>> + Send + Sync + 'a>,
    pub name: Cow<'a, str>,
}

impl Debug for Function<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{{function {}}}", self.name)
    }
}

impl PartialEq for Function<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Comparing only func pointers, names are decorative
        Arc::ptr_eq(&self.call, &other.call)
    }
}

impl Eq for Function<'_> {}
