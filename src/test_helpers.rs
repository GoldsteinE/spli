use crate::{
    parser::{IResult, Span},
    Value, ValueKind,
};
use std::fmt::Debug;

pub fn assert_ok_span(t1: IResult<Span>, t2: (Span, Span)) {
    let t1 = t1.unwrap();
    assert_eq!(t1.0.to_string(), t2.0.to_string());
    assert_eq!(t1.1.to_string(), t2.1.to_string());
}

pub fn assert_ok_t<T: PartialEq + Debug>(t1: IResult<T>, t2: (Span, T)) {
    let t1 = t1.unwrap();
    assert_eq!(t1.0.to_string(), t2.0.to_string());
    assert_eq!(t1.1, t2.1);
}

pub fn simple_value(kind: ValueKind) -> Value {
    Value {
        raw: false,
        sequential: false,
        kind,
    }
}

pub fn raw_value(kind: ValueKind) -> Value {
    Value {
        raw: true,
        sequential: false,
        kind,
    }
}

pub fn sequential_value(kind: ValueKind) -> Value {
    Value {
        raw: false,
        sequential: true,
        kind,
    }
}
