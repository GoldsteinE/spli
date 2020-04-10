use super::IResult;

const VALID_IDENT_PUNCT: &'static str = "+-*/.:^%&$#@";

fn is_valid_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || VALID_IDENT_PUNCT.contains(c)
}

fn is_valid_ident_char(c: char) -> bool {
    is_valid_ident_start(c) || c.is_ascii_digit()
}

pub fn ident(i: &str) -> IResult<&str> {
    nom::error::context(
        "symbol",
        nom::combinator::recognize(nom::sequence::tuple((
            nom::bytes::complete::take_while1(is_valid_ident_start),
            nom::bytes::complete::take_while(is_valid_ident_char),
        ))
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident() {
        assert_eq!(ident("name"), Ok(("", "name")));
        assert_eq!(ident("name and more"), Ok((" and more", "name")));
        assert_eq!(ident(VALID_IDENT_PUNCT), Ok(("", VALID_IDENT_PUNCT)));
        assert_eq!(ident("name'"), Ok(("'", "name")));
        assert!(ident("'invalid character").is_err());
    }
}
