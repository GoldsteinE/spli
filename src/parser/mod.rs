use crate::list::List;
use crate::{Value, ValueKind};

mod error_handling;
mod ident;
mod list;
mod number;
mod string;

pub use error_handling::determine_error;
pub use ident::ident;
pub use list::{list, token};
pub use number::{float, integer};
pub use string::string;

use nom::character::complete::{multispace0, multispace1};

pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;
pub type Error<'a> = nom_greedyerror::GreedyError<Span<'a>>;
pub type IResult<'a, O> = nom::IResult<Span<'a>, O, Error<'a>>;

pub fn program<'a>(i: Span<'a>) -> IResult<Value<'a>> {
    let (mut i, _) = multispace0(i)?;
    let mut result = Vec::new();
    let mut first_token = true;
    while i.fragment().len() != 0 {
        if !first_token {
            i = multispace1(i)?.0;
        } else {
            i = multispace0(i)?.0;
            first_token = false;
        }
        if i.fragment().len() == 0 {
            break;
        }
        let i_value = token(i)?;
        i = i_value.0;
        let value = i_value.1;
        result.push(value);
    }
    Ok((
        Span::new(""),
        Value {
            raw: false,
            sequential: true,
            kind: ValueKind::List(List::from_double_ended_iter(result)),
        },
    ))
}
