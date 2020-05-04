use super::{IResult, Span};
use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char as one_char, digit0, digit1, hex_digit1, oct_digit1, one_of},
    combinator::{map_res, recognize},
    multi::many1,
    sequence::{self, preceded},
};

fn decimal_integer(i: Span) -> IResult<i64> {
    map_res(
        alt((
            recognize(sequence::tuple((one_of("123456789"), digit0))),
            recognize(one_char('0')),
        )),
        |n: Span| i64::from_str(n.fragment()),
    )(i)
}

fn hex_integer(i: Span) -> IResult<i64> {
    map_res(preceded(tag("0x"), hex_digit1), |n: Span| {
        i64::from_str_radix(n.fragment(), 16)
    })(i)
}

fn oct_integer(i: Span) -> IResult<i64> {
    map_res(preceded(tag("0o"), oct_digit1), |n: Span| {
        i64::from_str_radix(n.fragment(), 8)
    })(i)
}

fn bin_integer(i: Span) -> IResult<i64> {
    map_res(
        preceded(tag("0b"), recognize(many1(one_of("01")))),
        |n: Span| i64::from_str_radix(n.fragment(), 2),
    )(i)
}

pub fn integer(i: Span) -> IResult<i64> {
    alt((hex_integer, bin_integer, oct_integer, decimal_integer))(i)
}

pub fn float(i: Span) -> IResult<f64> {
    map_res(
        recognize(sequence::tuple((digit1, one_char('.'), digit0))),
        |n: Span| f64::from_str(n.fragment()),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::assert_ok_t;

    #[test]
    fn test_integer() {
        assert_ok_t(integer(Span::new("0")), (Span::new(""), 0));
        assert_ok_t(integer(Span::new("123")), (Span::new(""), 123));
        assert_ok_t(integer(Span::new("0x123")), (Span::new(""), 0x123));
        assert_ok_t(integer(Span::new("0o123")), (Span::new(""), 0o123));
        assert_ok_t(integer(Span::new("0b1010")), (Span::new(""), 10));
        assert_ok_t(integer(Span::new("0b123")), (Span::new("23"), 1));
        assert_ok_t(integer(Span::new("0123")), (Span::new("123"), 0));
        assert_ok_t(integer(Span::new("0q123")), (Span::new("q123"), 0));
    }

    #[test]
    fn test_float() {
        assert_ok_t(float(Span::new("123.0")), (Span::new(""), 123.0));
        assert_ok_t(float(Span::new("5.6")), (Span::new(""), 5.6));
        assert_ok_t(float(Span::new("05.6")), (Span::new(""), 5.6));
        assert_ok_t(float(Span::new("5.6.")), (Span::new("."), 5.6));
        assert_ok_t(float(Span::new("5.")), (Span::new(""), 5.0));
        assert!(float(Span::new(".")).is_err());
        assert!(float(Span::new(".123")).is_err());
        assert!(float(Span::new("0x5.6")).is_err());
        assert!(float(Span::new("0o5.6")).is_err());
        assert!(float(Span::new("0b0.1")).is_err());
    }
}
