use snafu::prelude::*;
use winnow::{
    combinator::{alt, opt, repeat},
    error::{ContextError, ParseError},
    seq, Parser,
};

use crate::{BlockElement, Document};

use super::{
    comment::parse_comment, glyph_definition::parse_glyph_definition, property::parse_property,
};

#[derive(Snafu)]
#[snafu(display("parse error: {origin}"))]
pub struct YaffParseError<'a> {
    pub origin: ParseError<&'a str, ContextError>,
}

pub fn parse_document<'a>(input: &mut &'a str) -> Result<Document, YaffParseError<'a>> {
    seq!(Document::new(
        // A byte-order mark (u+FEFF) may be included at the start of the file.
        _: opt('\u{FEFF}'),
        repeat(1.., alt((
            parse_glyph_definition.map(BlockElement::GlyphDefinition),
            parse_property.map(BlockElement::Property),
            parse_comment.map(BlockElement::Comment),
        )))
    ))
    .parse(input)
    .map_err(|origin| YaffParseError { origin })
}
