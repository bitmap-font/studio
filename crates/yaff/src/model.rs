use snafu::prelude::*;
use std::num::TryFromIntError;

pub struct Document {
    elements: Vec<BlockElement>,
}

impl Document {
    pub fn new(elements: Vec<BlockElement>) -> Document {
        Document { elements }
    }
}

pub enum BlockElement {
    Comment(Comment),
    Whitespace(String),
    Property(Property),
    GlyphDefinition(GlyphDefinition),
}
pub struct Comment(pub String);

pub struct Property {
    pub key: String,
    pub value: String,
}

pub struct GlyphDefinition {
    pub labels: Vec<GlyphLabel>,
    pub indent: String,
    pub value: Option<GlyphValue>,
}

pub enum GlyphLabel {
    CodepointSingle(u32),
    CodepointSequence(Vec<u8>),
    CharacterSingle(char),
    CharacterSequence(Vec<char>),
    Tag(String),
}

#[derive(Clone)]
pub enum GlyphPaletteColor {
    Zero = 0x0,
    One = 0x1,
    Two = 0x2,
    Three = 0x3,
    Four = 0x4,
    Five = 0x5,
    Six = 0x6,
    Seven = 0x7,
    Eight = 0x8,
    Nine = 0x9,
    Ten = 0xA,
    Eleven = 0xB,
    Twelve = 0xC,
    Thirteen = 0xD,
    Fourteen = 0xE,
    Fifteen = 0xF,
}

impl GlyphPaletteColor {
    pub fn value(&self) -> u8 {
        match self {
            GlyphPaletteColor::Zero => 0x0,
            GlyphPaletteColor::One => 0x1,
            GlyphPaletteColor::Two => 0x2,
            GlyphPaletteColor::Three => 0x3,
            GlyphPaletteColor::Four => 0x4,
            GlyphPaletteColor::Five => 0x5,
            GlyphPaletteColor::Six => 0x6,
            GlyphPaletteColor::Seven => 0x7,
            GlyphPaletteColor::Eight => 0x8,
            GlyphPaletteColor::Nine => 0x9,
            GlyphPaletteColor::Ten => 0xA,
            GlyphPaletteColor::Eleven => 0xB,
            GlyphPaletteColor::Twelve => 0xC,
            GlyphPaletteColor::Thirteen => 0xD,
            GlyphPaletteColor::Fourteen => 0xE,
            GlyphPaletteColor::Fifteen => 0xF,
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(display("{ch} cannot be part of glyph"))]
pub struct InvalidGlyphPaletteColorError {
    ch: char,
}

impl GlyphPaletteColor {
    pub fn try_from(ch: char) -> Result<Option<GlyphPaletteColor>, InvalidGlyphPaletteColorError> {
        match ch {
            '0' | '@' => Ok(Some(GlyphPaletteColor::Zero)),
            '1' => Ok(Some(GlyphPaletteColor::One)),
            '2' => Ok(Some(GlyphPaletteColor::Two)),
            '3' => Ok(Some(GlyphPaletteColor::Three)),
            '4' => Ok(Some(GlyphPaletteColor::Four)),
            '5' => Ok(Some(GlyphPaletteColor::Five)),
            '6' => Ok(Some(GlyphPaletteColor::Six)),
            '7' => Ok(Some(GlyphPaletteColor::Seven)),
            '8' => Ok(Some(GlyphPaletteColor::Eight)),
            '9' => Ok(Some(GlyphPaletteColor::Nine)),
            'A' => Ok(Some(GlyphPaletteColor::Ten)),
            'B' => Ok(Some(GlyphPaletteColor::Eleven)),
            'C' => Ok(Some(GlyphPaletteColor::Twelve)),
            'D' => Ok(Some(GlyphPaletteColor::Thirteen)),
            'E' => Ok(Some(GlyphPaletteColor::Fourteen)),
            'F' => Ok(Some(GlyphPaletteColor::Fifteen)),
            '.' => Ok(None),
            ch => Err(InvalidGlyphPaletteColorError { ch }),
        }
    }
}

pub struct GlyphValue {
    pub width: usize,
    pub height: usize,
    data: Vec<Vec<Option<GlyphPaletteColor>>>,
}

#[derive(Debug, Snafu)]
#[snafu(display("row-length of glyph is not all same: {}", widths.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")))]
pub struct GlyphNotRectangleError {
    widths: Vec<usize>,
}

impl GlyphValue {
    pub fn new(
        data: Vec<Vec<Option<GlyphPaletteColor>>>,
    ) -> Result<GlyphValue, GlyphNotRectangleError> {
        let mut widths: Vec<_> = data.iter().map(|row| row.len()).collect();
        widths.sort();
        widths.dedup();
        match &widths[..] {
            &[width] => Ok(GlyphValue {
                width,
                height: data.len(),
                data,
            }),
            _ => Err(GlyphNotRectangleError { widths }),
        }
    }
}

impl GlyphLabel {
    pub fn try_from_codepoint(codepoints: Vec<u32>) -> Result<GlyphLabel, TryFromIntError> {
        if codepoints.len() == 1 {
            Ok(GlyphLabel::CodepointSingle(codepoints[0]))
        } else {
            codepoints
                .into_iter()
                .map(u8::try_from)
                .collect::<Result<_, _>>()
                .map(GlyphLabel::CodepointSequence)
        }
    }
    pub fn from_character(characters: Vec<char>) -> GlyphLabel {
        if characters.len() == 1 {
            GlyphLabel::CharacterSingle(characters[0])
        } else {
            GlyphLabel::CharacterSequence(characters)
        }
    }
}
