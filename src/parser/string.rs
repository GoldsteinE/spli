use super::{IResult, Span};

use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not},
    character::complete::char as one_char,
    combinator::value,
    error::{context, make_error, ErrorKind},
    sequence::delimited,
};

fn escape<'a>(val: &'static str, tag: char, i: Span<'a>) -> IResult<'a, &'a str> {
    value(val, one_char(tag))(i)
}

fn escape_nl(i: Span) -> IResult<&str> {
    escape("\n", 'n', i)
}

fn escape_tab(i: Span) -> IResult<&str> {
    escape("\t", 't', i)
}

fn escape_quote(i: Span) -> IResult<&str> {
    escape("\"", '"', i)
}

fn escape_backslash(i: Span) -> IResult<&str> {
    escape("\\", '\\', i)
}

fn invalid_escape(i: Span) -> IResult<&str> {
    Err(nom::Err::Error(make_error(i, ErrorKind::OneOf)))
}

fn string_inner(i: Span) -> IResult<String> {
    escaped_transform(
        is_not("\\\""),
        '\\',
        context(
            "escape",
            alt((
                escape_nl,
                escape_tab,
                escape_quote,
                escape_backslash,
                invalid_escape,
            )),
        ),
    )(i)
}

pub fn string(i: Span) -> IResult<String> {
    delimited(one_char('"'), string_inner, one_char('"'))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::assert_ok_t;

    #[test]
    fn test_string() {
        assert_ok_t(
            string(Span::new("\"Hello, world!\"")),
            (Span::new(""), "Hello, world!".into()),
        );
        assert_ok_t(
            string(Span::new("\"Hello, world!\", \"Also, other data\"")),
            (Span::new(", \"Also, other data\""), "Hello, world!".into()),
        );
        assert_ok_t(
            string(Span::new(
                "\"String with \\n multiple \\t escapes \\\" \\\\ :)\"",
            )),
            (
                Span::new(""),
                "String with \n multiple \t escapes \" \\ :)".into(),
            ),
        );
        assert!(string(Span::new("Unquoted string")).is_err());
        assert!(string(Span::new("\"Unterminated string")).is_err());
        assert!(string(Span::new("\"Badly terminated string\\\"")).is_err());
        assert!(string(Span::new("\"String with \\bad escape\"")).is_err());
    }
}
