use winnow::{
    combinator::{opt, repeat_till},
    seq,
    token::any,
    PResult, Parser,
};

use crate::{syntax::fragments::parse_line_terminator, Comment};

pub fn parse_comment(input: &mut &str) -> PResult<Comment> {
    seq!(Comment(
        _: '#',
        _: opt(' '),
        repeat_till(0.., any, parse_line_terminator).map(|(acc, _): (Vec<char>, _)| acc.into_iter().collect()))
    )
    .parse_next(input)
}
