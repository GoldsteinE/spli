use super::{Span, Error as ParsingError};
use nom::error::ErrorKind;
use nom_greedyerror::GreedyErrorKind;
use show_my_errors::{Annotation, Result};

fn unknown_error(err: &ParsingError) -> Result<Annotation> {
    let (span, kind) = match err.errors.iter().next() {
        Some(error) => error,
        None => unreachable!(),
    };
    let offset = span.location_offset();
    let message = format!("unknown parsing error: {:?}", kind);
    Annotation::error(offset..offset + 1, message, "somewhere here")
}

fn whitespace_error(offset: usize) -> Result<Annotation> {
    Annotation::error(offset..offset + 1, "expected whitespace after token", "here")
}

fn escape_context_error(span: &Span) -> Result<Annotation> {
    let offset = span.location_offset();
    Annotation::error(offset..offset + 1, "unknown escape code", None)
}

fn unclosed_list_error(span: &Span, err: &ParsingError) -> Result<Annotation> {
    let first_list = err.errors.iter().filter_map(|(span, kind)| {
        if let GreedyErrorKind::Context("list") = kind {
            Some(span)
        } else {
            None
        }
    }).next();
    if let Some(list_span) = first_list {
        let offset = list_span.location_offset();
        Annotation::error(offset..offset + 1, "unclosed list", "started here")
    } else {
        let offset = span.location_offset() - 1;
        Annotation::error(offset..offset, "unclosed list", None)
    }
}

fn invalid_ident_error(span: &Span) -> Result<Annotation> {
    let ident = span.fragment().split_ascii_whitespace().next().unwrap_or("");
    let offset = span.location_offset();
    Annotation::error(offset..offset + ident.chars().count(), "invalid identifier", None)
}

fn number_error(span: &Span) -> Result<Annotation> {
    let fragment = span.fragment();
    let number = fragment.split_ascii_whitespace().next().unwrap_or("");
    let offset = span.location_offset();
    Annotation::error(offset..offset + number.chars().count(), "invalid number", None)
}

fn string_error(span: &Span, err: &ParsingError) -> Result<Annotation> {
    match err.errors.first() {
        Some((_, GreedyErrorKind::Char('"'))) => {
            let offset = span.location_offset();
            Annotation::error(offset..offset + 1, "unclosed string", "started here")
        },
        None => unreachable!(),
        _ => unknown_error(err)
    }
}

fn list_error(source: &str, span: &Span, err: &ParsingError) -> Result<Annotation> {
    match err.errors.first() {
        Some((first_span, GreedyErrorKind::Nom(ErrorKind::MultiSpace))) => {
            if first_span.location_offset() == source.len() {
                unclosed_list_error(span, err)
            } else {
                whitespace_error(first_span.location_offset())
            }
        },
        None => unreachable!(),
        _ => unknown_error(err)
    }
}

pub fn determine_error(source: &str, err: &ParsingError) -> Result<Annotation> {
    let context = err.errors.iter().filter_map(|(span, kind)| {
        if let GreedyErrorKind::Context(context) = kind {
            Some((context, span))
        } else {
            None
        }
    }).next();

    match context {
        Some((&"escape", span)) => escape_context_error(span),
        Some((&"ident", span)) => {
            if span.fragment().len() == 0 && span.location_offset() == source.len() {
                unclosed_list_error(span, err)
            } else {
                invalid_ident_error(span)
            }
        },
        Some((&"number", span)) => number_error(span),
        Some((&"string", span)) => string_error(span, err),
        Some((&"list", span)) => list_error(source, span, err),
        None => {
            if let Some((span, GreedyErrorKind::Nom(ErrorKind::MultiSpace))) = err.errors.first() {
                whitespace_error(span.location_offset())
            } else {
                unknown_error(err)
            }
        }
        _ => unknown_error(err)
    }
}
