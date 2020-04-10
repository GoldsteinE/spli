use crate::{ValueKind, Value};
use crate::list::List;

mod ident;
mod number;
mod string;
mod list;

pub use ident::ident;
pub use number::{float, integer};
pub use string::string;
pub use list::{token, list};

type IResult<'a, O> = nom::IResult<&'a str, O, nom::error::VerboseError<&'a str>>;

pub fn program<'a>(i: &'a str) -> IResult<Value<'a>> {
    nom::combinator::map(
        nom::multi::separated_list(
            nom::character::complete::space1,
            token,
        ),
        |v| Value {
            raw: false,
            sequential: true,
            kind: ValueKind::List(List::from_double_ended_iter(v))
        }
    )(i)
}
