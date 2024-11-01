use snafu::prelude::*;
use winnow::{
    combinator::{alt, opt, repeat, separated_foldl1},
    error::ContextError,
    seq, Parser,
};

use crate::{BlockElement, Document};

use super::{
    comment::parse_comment,
    fragments::{parse_line_terminator, parse_whitespace},
    glyph_definition::parse_glyph_definition,
    property::parse_property,
};

#[derive(Debug, Snafu)]
#[snafu(display("parse error: {msg}"))]
pub struct YaffParseError {
    pub offset: usize,
    pub msg: String,
    pub origin: ContextError,
}

pub fn parse_document(input: &mut &str) -> Result<Document, YaffParseError> {
    seq!(Document::new(
        // A byte-order mark (u+FEFF) may be included at the start of the file.
        _: opt('\u{FEFF}'),
        repeat(0.., alt((
            parse_glyph_definition.map(BlockElement::GlyphDefinition),
            parse_property.map(BlockElement::Property),
            parse_comment.map(BlockElement::Comment),
            repeat(
                1..,
                alt((
                    parse_whitespace,
                    parse_line_terminator.verify_map(|opt| opt),
                ))
            ).map(|acc: Vec<_>| acc.join("")).verify(|s: &String| !s.is_empty()).map(BlockElement::Whitespace),
        )))
    ))
    .parse(input)
    .map_err(|e| YaffParseError {
        offset: e.offset(),
        msg: e.to_string(),
        origin: e.into_inner(),
    })
}
