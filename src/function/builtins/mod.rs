pub mod io;
pub mod lang;
pub mod list;
pub mod math;

use super::Function;

#[macro_export]
macro_rules! _count_idents {
    ($($idents:ident),*) => {{
        #[allow(dead_code, non_camel_case_types)]
        enum Idents { $($idents,)* __CountIdentsMacroLast }
        Idents::__CountIdentsMacroLast as usize
    }}
}

#[macro_export]
macro_rules! _count_patterns {
    () => { 0i64 };
    ($x:pat) => { 1i64 };
    ($x:pat, $($xs:pat),*) => { 1i64 + $crate::_count_patterns!($($xs),*) };
}

#[macro_export]
macro_rules! _parse_one_arg {
    ($val:expr, $count:expr, $var:pat, $kind_s:literal => $kind:path) => {
        $crate::_parse_one_arg!($val, $count, var);
        let $var = match &var.kind {
            $kind(val) => val,
            _ => return $crate::exception::Exception::wrong_type($kind_s, var).to_value(),
        };
    };
    ($val:expr, $count:expr, $var:pat) => {
        let $var = match $val {
            ::std::option::Option::Some(val) => val,
            ::std::option::Option::None => {
                return $crate::exception::Exception::too_few_arguments($count).to_value()
            }
        };
    };
}

#[macro_export]
macro_rules! _parse_rest {
    ($args:expr,) => {
        if $args.len() != 0 {
            return $crate::exception::Exception::too_many_arguments(VARS_COUNT).to_value();
        }
    };
    ($args:expr, $rest:pat) => {
        let $rest = $args;
    };
}

#[macro_export]
macro_rules! _parse_args {
    ($args:expr, { $(let $var:pat $(= $kind:path | $kind_s:literal)?),* $(, let @$rest:pat)? }) => {
        const VARS_COUNT: ::std::primitive::i64 = $crate::_count_patterns!($($var),*);
        $(
            $crate::_parse_one_arg!($args.pop(), VARS_COUNT, $var $(, $kind_s => $kind)?);
        )*
        $crate::_parse_rest!($args, $($rest)?);
    };
}

pub use crate::_parse_args as parse_args;

pub fn builtins<'a>() -> Vec<Function<'a>> {
    [
        lang::builtins(),
        list::builtins(),
        math::builtins(),
        io::builtins(),
    ]
    .concat()
}
