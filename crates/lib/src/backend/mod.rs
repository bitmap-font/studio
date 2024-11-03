use std::{error::Error, path::Path};

use yaff::GlyphDefinition;

mod opentype_ttf;

pub use opentype_ttf::{OpentypeTtfBackend, OpentypeTtfBuildError};

pub struct FontOptions {
    pub copyright_notice: Option<String>,
    pub family_name: String,
    pub sub_family_name: String,
    pub unique_id: String,
    pub full_font_name: Option<String>,
    pub postscript_name: Option<String>,

    pub version: FontVerseion,

    pub height: u16,
}

pub struct FontVerseion {
    pub(crate) major: u8,
    pub(crate) minor: u8,
    pub(crate) metadata: Option<String>,
}

impl FontVerseion {
    pub fn new(major: u8, minor: u8) -> Option<FontVerseion> {
        if minor >= 100 {
            return None;
        }
        Some(FontVerseion {
            major,
            minor,
            metadata: None,
        })
    }
    pub fn with_metadata(major: u8, minor: u8, metadata: impl AsRef<str>) -> Option<FontVerseion> {
        if minor >= 100 {
            return None;
        }
        Some(FontVerseion {
            major,
            minor,
            metadata: Some(metadata.as_ref().to_owned()),
        })
    }
}

pub trait FontBackend {
    type Err: Error;

    fn add_glyph(&mut self, glyph: &GlyphDefinition);

    fn build_to(self, dir: impl AsRef<Path>) -> Result<(), Self::Err>;
}
