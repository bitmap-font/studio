use winnow::{
    combinator::{alt, opt, preceded, repeat, repeat_till},
    seq,
    token::{any, none_of, one_of},
    PResult, Parser,
};

use crate::{syntax::fragments::parse_whitespace, Property};

use super::fragments::parse_line_terminator;

pub fn parse_property(input: &mut &str) -> PResult<Property> {
    seq!(Property {
        key: repeat(1.., parse_property_key_part).map(|acc: Vec<_>| acc.into_iter().collect()),
        _: opt(parse_whitespace),
        _: ':',
        _: opt(parse_whitespace),
        value: preceded(
            parse_whitespace,
            alt((
                preceded(parse_line_terminator, parse_multiline_value),
                parse_singleline_value,
            ))
        )
    })
    .parse_next(input)
}

fn parse_property_key_part(input: &mut &str) -> PResult<char> {
    one_of(('a'..='z', 'A'..='Z', '0'..='9', '_', '-', '.')).parse_next(input)
}

fn parse_multiline_value(input: &mut &str) -> PResult<String> {
    repeat(
        0..,
        seq!(
            _: parse_whitespace,
            none_of((':', '.', '@')),
            repeat_till(1.., any, seq!(
                opt(parse_whitespace),
                parse_line_terminator,
            )).map(|(acc, _): (Vec<_>, _)| -> String {acc.into_iter().collect()}),
        )
        .map(|(start, cont)| format!("{start}{cont}"))
        .verify(|s: &str| s.chars().any(|c| c != '-'))
        .map(|s: String| {
            if s.starts_with('"') && s.ends_with('"') {
                s[1..s.len() - 1].to_owned()
            } else {
                s
            }
        }),
    )
    .map(|acc: Vec<_>| acc.join("\n"))
    .parse_next(input)
}

fn parse_singleline_value(input: &mut &str) -> PResult<String> {
    repeat_till(0.., any, seq!(parse_whitespace, parse_line_terminator))
        .map(|(acc, _): (Vec<_>, _)| acc.into_iter().collect())
        .parse_next(input)
}
