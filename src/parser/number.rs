use std::str::FromStr;
use super::IResult;

fn decimal_integer(i: &str) -> IResult<i64> {
    nom::combinator::map_res(
        nom::combinator::recognize(nom::sequence::tuple((
            nom::character::complete::one_of("123456789"),
            nom::character::complete::digit0,
        ))),
        i64::from_str,
    )(i)
}

fn hex_integer(i: &str) -> IResult<i64> {
    nom::combinator::map_res(
        nom::sequence::preceded(
            nom::bytes::complete::tag("0x"),
            nom::character::complete::hex_digit1,
        ),
        |n| i64::from_str_radix(n, 16),
    )(i)
}

fn oct_integer(i: &str) -> IResult<i64> {
    nom::combinator::map_res(
        nom::sequence::preceded(
            nom::bytes::complete::tag("0o"),
            nom::character::complete::oct_digit1,
        ),
        |n| i64::from_str_radix(n, 8),
    )(i)
}

fn bin_integer(i: &str) -> IResult<i64> {
    nom::combinator::map_res(
        nom::sequence::preceded(
            nom::bytes::complete::tag("0b"),
            nom::combinator::recognize(nom::multi::many1(nom::character::complete::one_of("01"))),
        ),
        |n| i64::from_str_radix(n, 2),
    )(i)
}

pub fn integer(i: &str) -> IResult<i64> {
    nom::error::context(
        "integer",
        nom::branch::alt((decimal_integer, hex_integer, bin_integer, oct_integer))
    )(i)
}

pub fn float(i: &str) -> IResult<f64> {
    nom::error::context(
        "float",
        nom::combinator::map_res(
            nom::combinator::recognize(nom::branch::alt((
                nom::sequence::tuple((
                    nom::character::complete::digit0,
                    nom::character::complete::char('.'),
                    nom::character::complete::digit1,
                )),
                nom::sequence::tuple((
                    nom::character::complete::digit1,
                    nom::character::complete::char('.'),
                    nom::character::complete::digit0,
                )),
            ))),
            f64::from_str,
        )
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer() {
        assert_eq!(integer("123"), Ok(("", 123)));
        assert_eq!(integer("0x123"), Ok(("", 0x123)));
        assert_eq!(integer("0o123"), Ok(("", 0o123)));
        assert_eq!(integer("0b1010"), Ok(("", 10)));
        assert_eq!(integer("0b123"), Ok(("23", 1)));
        assert!(integer("0123").is_err());
        assert!(integer("0q123").is_err());
    }

    #[test]
    fn test_float() {
        assert_eq!(float("123.0"), Ok(("", 123.0)));
        assert_eq!(float(".123"), Ok(("", 0.123)));
        assert_eq!(float("5.6"), Ok(("", 5.6)));
        assert_eq!(float("05.6"), Ok(("", 5.6)));
        assert_eq!(float("5.6."), Ok((".", 5.6)));
        assert_eq!(float("5."), Ok(("", 5.0)));
        assert!(float(".").is_err());
        assert!(float("0x5.6").is_err());
        assert!(float("0o5.6").is_err());
        assert!(float("0b0.1").is_err());
    }
}
