use crate::parser::{IResult, Span};
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
