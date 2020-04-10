#![allow(unused_imports)]

use crate::{list, list::List};
use crate::{Value, ValueKind};

use super::{IResult, float, ident, integer, string};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Modifier {
    Raw,
    Sequential,
    None,
}

fn modifier(i: &str) -> IResult<Modifier> {
    nom::combinator::map(
        nom::branch::alt((
            nom::character::complete::char('\''),
            nom::character::complete::char('!'),
            nom::combinator::value('\0', nom::bytes::complete::take(0usize)),
        )),
        |c| match c {
            '\'' => Modifier::Raw,
            '!' => Modifier::Sequential,
            _ => Modifier::None,
        },
    )(i)
}

pub fn token<'a>(i: &'a str) -> IResult<Value<'a>> {
    nom::combinator::map(
        nom::sequence::tuple((
            modifier,
            nom::branch::alt((
                nom::combinator::map(string, ValueKind::String),
                nom::combinator::map(ident, ValueKind::Symbol),
                nom::combinator::map(float, ValueKind::Float),
                nom::combinator::map(integer, ValueKind::Integer),
                nom::combinator::map(list, ValueKind::List),
            )),
        )),
        |(modifier, kind)| Value {
            kind,
            raw: (modifier == Modifier::Raw),
            sequential: (modifier == Modifier::Sequential),
        },
    )(i)
}

pub fn list<'a>(i: &'a str) -> IResult<List<Value<'a>>> {
    nom::error::context(
        "list",
        nom::sequence::delimited(
            nom::character::complete::char('('),
            nom::combinator::map(
                nom::multi::separated_list(
                    nom::character::complete::space1,
                    token,
                ),
                List::from_double_ended_iter,
            ),
            nom::character::complete::char(')'),
        )
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Value,
        ValueKind::{self, *},
    };

    fn simple_value(kind: ValueKind) -> Value {
        Value {
            raw: false,
            sequential: false,
            kind,
        }
    }

    fn raw_value(kind: ValueKind) -> Value {
        Value {
            raw: true,
            sequential: false,
            kind,
        }
    }

    fn sequential_value(kind: ValueKind) -> Value {
        Value {
            raw: false,
            sequential: true,
            kind,
        }
    }

    #[test]
    fn test_token() {
        assert_eq!(token("1").unwrap(), ("", simple_value(Integer(1))));
        assert_eq!(token("1.2").unwrap(), ("", simple_value(Float(1.2))));
        assert_eq!(
            token("\"3\"").unwrap(),
            ("", simple_value(String("3".into())))
        );
        assert_eq!(token("four").unwrap(), ("", simple_value(Symbol("four"))));
        assert_eq!(token("'four").unwrap(), ("", raw_value(Symbol("four"))));
        assert_eq!(
            token("'(1 2 3)").unwrap(),
            (
                "",
                raw_value(List(list![
                    simple_value(Integer(1)),
                    simple_value(Integer(2)),
                    simple_value(Integer(3)),
                ]))
            )
        );
        assert_eq!(
            token("!(1 2 3)").unwrap(),
            (
                "",
                sequential_value(List(list![
                    simple_value(Integer(1)),
                    simple_value(Integer(2)),
                    simple_value(Integer(3)),
                ]))
            )
        );
    }

    #[test]
    fn test_simple_list() {
        assert_eq!(
            list("(1 2 3)").unwrap(),
            (
                "",
                list![
                    simple_value(Integer(1)),
                    simple_value(Integer(2)),
                    simple_value(Integer(3)),
                ]
            )
        );
    }

    #[test]
    fn test_heterogenous_list() {
        assert_eq!(
            list("(1 1.2 \"3\" four)").unwrap(),
            (
                "",
                list![
                    simple_value(Integer(1)),
                    simple_value(Float(1.2)),
                    simple_value(String("3".into())),
                    simple_value(Symbol("four"))
                ]
            )
        );
    }

    #[test]
    fn test_after_list() {
        assert_eq!(
            list("(1 2 3)4").unwrap(),
            (
                "4",
                list![
                    simple_value(Integer(1)),
                    simple_value(Integer(2)),
                    simple_value(Integer(3)),
                ]
            )
        );
    }

    #[test]
    fn test_nested_lists() {
        assert_eq!(
            list("(+ !(/ 2 3) (eval '(* 2 4)) 6)").unwrap(),
            (
                "",
                list![
                    simple_value(Symbol("+")),
                    sequential_value(List(list![
                        simple_value(Symbol("/")),
                        simple_value(Integer(2)),
                        simple_value(Integer(3)),
                    ])),
                    simple_value(List(list![
                        simple_value(Symbol("eval")),
                        raw_value(List(list![
                            simple_value(Symbol("*")),
                            simple_value(Integer(2)),
                            simple_value(Integer(4)),
                        ]))
                    ])),
                    simple_value(Integer(6))
                ]
            )
        );
    }
}
