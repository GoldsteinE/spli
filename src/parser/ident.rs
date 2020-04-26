use super::{IResult, Span};

use nom::{
    bytes::complete::{take_while, take_while1},
    combinator::recognize,
    sequence,
};

const VALID_IDENT_PUNCT: &'static str = "+-*/.:^%&$#@";

fn is_valid_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || VALID_IDENT_PUNCT.contains(c)
}

fn is_valid_ident_char(c: char) -> bool {
    is_valid_ident_start(c) || c.is_ascii_digit()
}

pub fn ident(i: Span) -> IResult<Span> {
    recognize(sequence::tuple((
        take_while1(is_valid_ident_start),
        take_while(is_valid_ident_char),
    )))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::assert_ok_span;

    #[test]
    fn test_ident() {
        assert_ok_span(ident(Span::new("name")), (Span::new(""), Span::new("name")));
        assert_ok_span(
            ident(Span::new("name and more")),
            (Span::new(" and more"), Span::new("name")),
        );
        assert_ok_span(
            ident(Span::new(VALID_IDENT_PUNCT)),
            (Span::new(""), Span::new(VALID_IDENT_PUNCT)),
        );
        assert_ok_span(
            ident(Span::new("name'")),
            (Span::new("'"), Span::new("name")),
        );
        assert!(ident(Span::new("'invalid character")).is_err());
    }
}
