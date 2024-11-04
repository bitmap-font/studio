use core::fmt;
use snafu::prelude::*;
use std::{collections::HashMap, fmt::Write, num::TryFromIntError};

pub struct Document {
    elements: Vec<BlockElement>,
    glyph_lut: HashMap<SemanticGlyphLabel, usize>,
}

impl Document {
    pub fn new(elements: Vec<BlockElement>) -> Document {
        let mut glyph_lut = HashMap::new();
        for (idx, e) in elements.iter().enumerate() {
            match e {
                BlockElement::Comment(_) => {}
                BlockElement::Whitespace(_) => {}
                BlockElement::Property(_) => {}
                BlockElement::GlyphDefinition(glyph) => {
                    for label in glyph.labels.iter().flat_map(|label| label.to_semantic()) {
                        glyph_lut.insert(label, idx);
                    }
                }
            }
        }
        Document {
            elements,
            glyph_lut,
        }
    }

    pub fn get_glyph(&self, label: &SemanticGlyphLabel) -> Option<&GlyphDefinition> {
        let idx = self.glyph_lut.get(label)?;
        match self.elements.get(*idx) {
            Some(BlockElement::GlyphDefinition(def)) => Some(def),
            _ => None,
        }
    }

    pub fn list_glyph(&self) -> impl Iterator<Item = &GlyphDefinition> {
        self.glyph_lut
            .values()
            .flat_map(|idx| match &self.elements.get(*idx) {
                Some(BlockElement::GlyphDefinition(def)) => Some(def),
                _ => None,
            })
    }
}

#[derive(Debug)]
pub enum BlockElement {
    Comment(Comment),
    Whitespace(String),
    Property(Property),
    GlyphDefinition(GlyphDefinition),
}

#[derive(Debug)]
pub struct Comment(pub String);

#[derive(Debug)]
pub struct Property {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct GlyphDefinition {
    pub labels: Vec<GlyphLabel>,
    pub indent: String,
    pub value: Option<GlyphValue>,
}

/// It is generally means 4-bit colors described as following table:
///
/// |   ID | Color   |   ID | Color          |
/// | ---: | :------ | ---: | :------------- |
/// |  `0` | Black*  |  `8` | Bright Black*  |
/// |  `1` | Red     |  `9` | Bright Red     |
/// |  `2` | Green   | `10` | Bright Green   |
/// |  `3` | Yellow  | `11` | Bright Yellow  |
/// |  `4` | Blue    | `12` | Bright Blue    |
/// |  `5` | Magenta | `13` | Bright Magenta |
/// |  `6` | Cyan    | `14` | Bright Cyan    |
/// |  `7` | White*  | `15` | Bright White*  |
///
/// But for hand-drawn anti-aliased fonts, we treat 0=Black, 7=White, 8=BrightBlack, 15=BrightWhite a bit differently if COLR flag is not manually set.
///
/// |   ID | Meanning           |
/// | ---: | :----------------- |
/// |  `0` | FG w/ 100% opacity |
/// |  `8` | FG w/  75% opacity |
/// |  `7` | FG w/  50% opacity |
/// | `15` | FG w/  25% opacity |
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct GlyphValue {
    pub width: u16,
    pub height: u16,
    pub data: Vec<Vec<Option<GlyphPaletteColor>>>,
}

#[derive(Debug, Snafu)]
#[snafu(display("row-length of glyph is not all same: {}", widths.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")))]
pub struct GlyphNotRectangleError {
    widths: Vec<u16>,
}

impl GlyphValue {
    pub fn new(
        data: Vec<Vec<Option<GlyphPaletteColor>>>,
    ) -> Result<GlyphValue, GlyphNotRectangleError> {
        let mut widths: Vec<_> = data.iter().map(|row| row.len() as u16).collect();
        widths.sort();
        widths.dedup();
        match &widths[..] {
            &[width] => Ok(GlyphValue {
                width,
                height: data.len() as u16,
                data,
            }),
            _ => Err(GlyphNotRectangleError { widths }),
        }
    }
}

#[derive(Debug)]
pub enum GlyphLabel {
    CodepointSingle(u32),
    CodepointSequence(Vec<u8>),
    CharacterSingle(char),
    CharacterSequence(Vec<char>),
    Tag(String),
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

    pub fn to_semantic(&self) -> Option<SemanticGlyphLabel> {
        Some(match self {
            GlyphLabel::CodepointSingle(codepoint) => {
                SemanticGlyphLabel::CharSequence(vec![char::from_u32(*codepoint)?])
            }
            GlyphLabel::CodepointSequence(vec) => SemanticGlyphLabel::CharSequence(
                vec.iter().map(|codepoint| *codepoint as char).collect(),
            ),
            GlyphLabel::CharacterSingle(ch) => SemanticGlyphLabel::CharSequence(vec![*ch]),
            GlyphLabel::CharacterSequence(vec) => SemanticGlyphLabel::CharSequence(vec.clone()),
            GlyphLabel::Tag(tag) => SemanticGlyphLabel::Tag(tag.clone()),
        })
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum SemanticGlyphLabel {
    CharSequence(Vec<char>),
    Tag(String),
}

impl fmt::Display for SemanticGlyphLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticGlyphLabel::CharSequence(vec) => f.write_str(&String::from_iter(vec)),
            SemanticGlyphLabel::Tag(tag) => f.write_fmt(format_args!("`{tag}`")),
        }
    }
}
