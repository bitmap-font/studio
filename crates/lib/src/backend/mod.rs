use std::{collections::HashMap, error::Error, path::Path};

use yaff::{GlyphDefinition, SemanticGlyphLabel};

mod opentype_ttf;

pub use opentype_ttf::{OpentypeTtfBackend, OpentypeTtfBuildError};

pub struct FontOptions {
    pub name: String,
    pub revision: f64,

    pub height: u16,
}

pub trait FontBackend {
    type Err: Error;

    fn new(options: FontOptions) -> Self;

    fn add_glyphs(&mut self, map: HashMap<SemanticGlyphLabel, GlyphDefinition>);

    fn build_to(self, dir: impl AsRef<Path>) -> Result<(), Self::Err>;
}
