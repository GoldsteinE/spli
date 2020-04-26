#![allow(unused_imports)]

use crate::{list, list::List};
use crate::{Value, ValueKind};

use super::{float, ident, integer, string, Error, IResult, Span};

use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{char as one_char, digit1, multispace0, multispace1},
    combinator::{map, peek, value},
    error::context,
    multi::separated_list,
    sequence::{self, delimited, preceded},
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Modifier {
    Raw,
    Sequential,
    None,
}

fn modifier(i: Span) -> IResult<Modifier> {
    map(
        alt((one_char('\''), one_char('!'), value('\0', take(0usize)))),
        |c| match c {
            '\'' => Modifier::Raw,
            '!' => Modifier::Sequential,
            _ => Modifier::None,
        },
    )(i)
}

fn token_kind<'a>(i: Span<'a>) -> IResult<ValueKind<'a>> {
    if peek::<_, _, Error<'a>, _>(one_char('"'))(i).is_ok() {
        context("string", map(string, ValueKind::String))(i)
    } else if peek::<_, _, Error<'a>, _>(one_char('('))(i).is_ok() {
        context("list", map(list, ValueKind::List))(i)
    } else if peek::<_, _, Error<'a>, _>(digit1)(i).is_ok() {
        context("number",
            alt((
                map(float, ValueKind::Float),
                map(integer, ValueKind::Integer),
            ))
        )(i)
    } else {
        context("ident", map(ident, |s| ValueKind::Symbol(s.fragment())))(i)
    }
}

pub fn token<'a>(i: Span<'a>) -> IResult<Value<'a>> {
    map(
        sequence::tuple((modifier, token_kind)),
        |(modifier, kind)| Value {
            kind,
            raw: (modifier == Modifier::Raw),
            sequential: (modifier == Modifier::Sequential),
        },
    )(i)
}

pub fn list<'a>(i: Span<'a>) -> IResult<List<Value<'a>>> {
    let mut result = Vec::new();
    let mut first_token = true;
    let (mut i, _) = one_char('(')(i)?;
    loop {
        if let Ok((i, _)) = preceded(multispace0, one_char::<_, Error>(')'))(i) {
            break Ok((i, List::from_double_ended_iter(result)));
        }
        if !first_token {
            i = multispace1(i)?.0;
        } else {
            i = multispace0(i)?.0;
            first_token = false;
        }
        let (new_i, token) = token(i)?;
        i = new_i;
        result.push(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_helpers::assert_ok_t,
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
        assert_ok_t(
            token(Span::new("1")),
            (Span::new(""), simple_value(Integer(1))),
        );
        assert_ok_t(
            token(Span::new("1.2")),
            (Span::new(""), simple_value(Float(1.2))),
        );
        assert_ok_t(
            token(Span::new("\"3\"")),
            (Span::new(""), simple_value(String("3".into()))),
        );
        assert_ok_t(
            token(Span::new("four")),
            (Span::new(""), simple_value(Symbol("four"))),
        );
        assert_ok_t(
            token(Span::new("'four")),
            (Span::new(""), raw_value(Symbol("four"))),
        );
        assert_ok_t(
            token(Span::new("'(1 2 3)")),
            (
                Span::new(""),
                raw_value(List(list![
                    simple_value(Integer(1)),
                    simple_value(Integer(2)),
                    simple_value(Integer(3)),
                ])),
            ),
        );
        assert_ok_t(
            token(Span::new("!(1 2 3)")),
            (
                Span::new(""),
                sequential_value(List(list![
                    simple_value(Integer(1)),
                    simple_value(Integer(2)),
                    simple_value(Integer(3)),
                ])),
            ),
        );
    }

    #[test]
    fn test_simple_list() {
        assert_ok_t(
            list(Span::new("(1 2 3)")),
            (
                Span::new(""),
                list![
                    simple_value(Integer(1)),
                    simple_value(Integer(2)),
                    simple_value(Integer(3)),
                ],
            ),
        );
    }

    #[test]
    fn test_heterogenous_list() {
        assert_ok_t(
            list(Span::new("(1 1.2 \"3\" four)")),
            (
                Span::new(""),
                list![
                    simple_value(Integer(1)),
                    simple_value(Float(1.2)),
                    simple_value(String("3".into())),
                    simple_value(Symbol("four"))
                ],
            ),
        );
    }

    #[test]
    fn test_after_list() {
        assert_ok_t(
            list(Span::new("(1 2 3)4")),
            (
                Span::new("4"),
                list![
                    simple_value(Integer(1)),
                    simple_value(Integer(2)),
                    simple_value(Integer(3)),
                ],
            ),
        );
    }

    #[test]
    fn test_nested_lists() {
        assert_ok_t(
            list(Span::new("(+ !(/ 2 3) (eval '(* 2 4)) 6)")),
            (
                Span::new(""),
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
                ],
            ),
        );
    }

    #[test]
    fn test_space_before_closing_paren() {
        assert_ok_t(
            list(Span::new("(a )")),
            (
                Span::new(""),
                list![
                    simple_value(Symbol("a"))
                ]
            )
        );
    }

    #[test]
    fn test_space_before_first_token() {
        assert_ok_t(
            list(Span::new("( a)")),
            (
                Span::new(""),
                list![
                    simple_value(Symbol("a"))
                ]
            )
        );
    }
}
