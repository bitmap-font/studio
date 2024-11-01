use winnow::{
    combinator::{alt, eof, repeat},
    token::one_of,
    PResult, Parser,
};

pub fn parse_line_terminator(input: &mut &str) -> PResult<Option<String>> {
    alt((
        '\n'.map(|ch| Some(ch.to_string())),
        '\r'.map(|ch| Some(ch.to_string())),
        "\r\n".map(|s: &str| Some(s.to_owned())),
        eof.map(|_| None),
    ))
    .parse_next(input)
}

pub fn parse_whitespace(input: &mut &str) -> PResult<String> {
    repeat(1.., one_of((' ', '\t'))).parse_next(input)
}
