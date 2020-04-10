use super::IResult;

fn escape<'a>(value: &'static str, tag: char, i: &'a str) -> IResult<'a, &'a str> {
    nom::combinator::value(value, nom::character::complete::char(tag))(i)
}

fn escape_nl(i: &str) -> IResult<&str> {
    escape("\n", 'n', i)
}

fn escape_tab(i: &str) -> IResult<&str> {
    escape("\t", 't', i)
}

fn escape_quote(i: &str) -> IResult<&str> {
    escape("\"", '"', i)
}

fn escape_backslash(i: &str) -> IResult<&str> {
    escape("\\", '\\', i)
}

fn string_inner(i: &str) -> IResult<String> {
    nom::bytes::complete::escaped_transform(
        nom::bytes::complete::is_not("\\\""),
        '\\',
        nom::branch::alt((escape_nl, escape_tab, escape_quote, escape_backslash)),
    )(i)
}

pub fn string(i: &str) -> IResult<String> {
    nom::error::context(
        "string",
        nom::sequence::delimited(
            nom::character::complete::char('"'),
            string_inner,
            nom::character::complete::char('"'),
        )
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        assert_eq!(
            string("\"Hello, world!\""),
            Ok(("", "Hello, world!".into()))
        );
        assert_eq!(
            string("\"Hello, world!\", \"Also, other data\""),
            Ok((", \"Also, other data\"", "Hello, world!".into()))
        );
        assert_eq!(
            string("\"String with \\n multiple \\t escapes \\\" \\\\ :)\""),
            Ok(("", "String with \n multiple \t escapes \" \\ :)".into()))
        );
        assert!(string("Unquoted string").is_err());
        assert!(string("\"Unterminated string").is_err());
        assert!(string("\"Badly terminated string\\\"").is_err());
        assert!(string("\"String with \\bad escape\"").is_err());
    }
}
